import type {
	DeviceState,
	AutoTighteningRequest,
	MultiSpindleConfigRequest,
	TighteningRequest,
	Pset
} from '$lib/types';

const API_BASE = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8081';

export class ApiClient {
	private async request<T>(endpoint: string, options?: RequestInit): Promise<T> {
		const response = await fetch(`${API_BASE}${endpoint}`, {
			headers: {
				'Content-Type': 'application/json',
				...options?.headers
			},
			...options
		});

		if (!response.ok) {
			throw new Error(`API error: ${response.status} ${response.statusText}`);
		}

		return response.json();
	}

	// GET /state
	async getDeviceState() {
		return this.request<DeviceState>('/state');
	}

	// POST /simulate/tightening
	async simulateTightening(payload: TighteningRequest = {}) {
		return this.request('/simulate/tightening', {
			method: 'POST',
			body: JSON.stringify(payload)
		});
	}

	// POST /auto-tightening/start
	async startAutoTightening(config: AutoTighteningRequest = {}) {
		return this.request('/auto-tightening/start', {
			method: 'POST',
			body: JSON.stringify(config)
		});
	}

	// POST /auto-tightening/stop
	async stopAutoTightening() {
		return this.request('/auto-tightening/stop', {
			method: 'POST'
		});
	}

	// GET /auto-tightening/status
	async getAutoTighteningStatus() {
		return this.request<{ running: boolean; counter: number; target_size: number; remaining_bolts: number }>('/auto-tightening/status');
	}

	// POST /config/multi-spindle
	async configureMultiSpindle(config: MultiSpindleConfigRequest) {
		return this.request('/config/multi-spindle', {
			method: 'POST',
			body: JSON.stringify(config)
		});
	}

	// GET /psets
	async getPsets() {
		return this.request<Pset[]>('/psets');
	}

	// GET /psets/:id
	async getPsetById(id: number) {
		return this.request<Pset>(`/psets/${id}`);
	}

	// POST /psets/:id/select
	async selectPset(id: number) {
		return this.request(`/psets/${id}/select`, {
			method: 'POST'
		});
	}

	// POST /psets - create new PSET
	async createPset(pset: Omit<Pset, 'id'>) {
		return this.request<{ success: boolean; message: string; pset: Pset }>('/psets', {
			method: 'POST',
			body: JSON.stringify({ ...pset, id: 0 })
		});
	}

	// PUT /psets/:id - update PSET
	async updatePset(id: number, pset: Pset) {
		return this.request<{ success: boolean; message: string; pset: Pset }>(`/psets/${id}`, {
			method: 'PUT',
			body: JSON.stringify(pset)
		});
	}

	// DELETE /psets/:id - delete PSET
	async deletePset(id: number) {
		return this.request<{ success: boolean; message: string }>(`/psets/${id}`, {
			method: 'DELETE'
		});
	}
}

export const api = new ApiClient();
