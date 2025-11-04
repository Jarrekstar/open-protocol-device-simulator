import type {
	DeviceState,
	AutoTighteningRequest,
	MultiSpindleConfigRequest,
	TighteningRequest,
	FailureConfig,
	FailureConfigRequest,
	Pset
} from '$lib/types';

const API_BASE = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8081';

/**
 * API client for communicating with the device simulator backend
 */
export class ApiClient {
	/**
	 * Internal method for making HTTP requests to the API
	 * @param endpoint - API endpoint path (e.g., '/state')
	 * @param options - Fetch options (method, headers, body, etc.)
	 * @returns Parsed JSON response
	 * @throws Error if the response status is not OK
	 */
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

	/**
	 * Retrieves the current device state
	 * @returns Device state with cell_id, tool status, PSET info, etc.
	 */
	async getDeviceState() {
		return this.request<DeviceState>('/state');
	}

	/**
	 * Simulates a single tightening operation
	 * @param payload - Tightening parameters (torque, angle, PSET override, etc.)
	 * @returns Response from the tightening simulation
	 */
	async simulateTightening(payload: TighteningRequest = {}) {
		return this.request('/simulate/tightening', {
			method: 'POST',
			body: JSON.stringify(payload)
		});
	}

	/**
	 * Starts automatic tightening mode with specified configuration
	 * @param config - Auto-tightening configuration (batch size, interval, etc.)
	 * @returns Response from starting auto-tightening
	 */
	async startAutoTightening(config: AutoTighteningRequest = {}) {
		return this.request('/auto-tightening/start', {
			method: 'POST',
			body: JSON.stringify(config)
		});
	}

	/**
	 * Stops the currently running automatic tightening mode
	 * @returns Response from stopping auto-tightening
	 */
	async stopAutoTightening() {
		return this.request('/auto-tightening/stop', {
			method: 'POST'
		});
	}

	/**
	 * Retrieves the current status of automatic tightening mode
	 * @returns Status with running state, counter, target size, and remaining bolts
	 */
	async getAutoTighteningStatus() {
		return this.request<{ running: boolean; counter: number; target_size: number; remaining_bolts: number }>('/auto-tightening/status');
	}

	/**
	 * Configures multi-spindle settings for the device
	 * @param config - Multi-spindle configuration
	 * @returns Response from configuring multi-spindle
	 */
	async configureMultiSpindle(config: MultiSpindleConfigRequest) {
		return this.request('/config/multi-spindle', {
			method: 'POST',
			body: JSON.stringify(config)
		});
	}

	/**
	 * Retrieves current failure injection configuration
	 * @returns Failure configuration with connection health and advanced settings
	 */
	async getFailureConfig() {
		return this.request<FailureConfig>('/config/failure');
	}

	/**
	 * Updates failure injection configuration for testing communication issues
	 * @param config - Failure configuration to apply
	 * @returns Response with success status and updated configuration
	 */
	async updateFailureConfig(config: FailureConfigRequest) {
		return this.request<{ success: boolean; message: string; config: FailureConfig }>('/config/failure', {
			method: 'POST',
			body: JSON.stringify(config)
		});
	}

	/**
	 * Retrieves all available PSETs
	 * @returns Array of PSET configurations
	 */
	async getPsets() {
		return this.request<Pset[]>('/psets');
	}

	/**
	 * Retrieves a specific PSET by ID
	 * @param id - PSET ID to retrieve
	 * @returns PSET configuration
	 */
	async getPsetById(id: number) {
		return this.request<Pset>(`/psets/${id}`);
	}

	/**
	 * Selects a PSET to use for tightening operations
	 * @param id - PSET ID to select
	 * @returns Response from selecting the PSET
	 */
	async selectPset(id: number) {
		return this.request(`/psets/${id}/select`, {
			method: 'POST'
		});
	}

	/**
	 * Creates a new PSET
	 * @param pset - PSET configuration without ID (ID will be auto-generated)
	 * @returns Response with success status and created PSET
	 */
	async createPset(pset: Omit<Pset, 'id'>) {
		return this.request<{ success: boolean; message: string; pset: Pset }>('/psets', {
			method: 'POST',
			body: JSON.stringify({ ...pset, id: 0 })
		});
	}

	/**
	 * Updates an existing PSET
	 * @param id - PSET ID to update
	 * @param pset - Updated PSET configuration
	 * @returns Response with success status and updated PSET
	 */
	async updatePset(id: number, pset: Pset) {
		return this.request<{ success: boolean; message: string; pset: Pset }>(`/psets/${id}`, {
			method: 'PUT',
			body: JSON.stringify(pset)
		});
	}

	/**
	 * Deletes a PSET
	 * @param id - PSET ID to delete
	 * @returns Response with success status
	 */
	async deletePset(id: number) {
		return this.request<{ success: boolean; message: string }>(`/psets/${id}`, {
			method: 'DELETE'
		});
	}
}

export const api = new ApiClient();
