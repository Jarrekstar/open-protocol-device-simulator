/**
 * PSET-related utility functions
 */

import type { Pset } from '$lib/types';

/**
 * Calculates the target value (midpoint) between min and max
 * @param min - The minimum value
 * @param max - The maximum value
 * @param decimals - Number of decimal places to round to
 * @returns The target value as a string
 */
export function calculateTarget(min: number, max: number, decimals: number = 1): string {
	return ((min + max) / 2).toFixed(decimals);
}

/**
 * Gets the target torque and angle for a PSET
 * @param pset - The PSET to calculate targets for
 * @returns An object with torque and angle targets
 */
export function getPsetTargets(
	pset: Pset
): { torque: string; angle: string } | null {
	if (!pset) return null;

	return {
		torque: calculateTarget(pset.torque_min, pset.torque_max, 1),
		angle: calculateTarget(pset.angle_min, pset.angle_max, 1)
	};
}

/**
 * Validates PSET values
 * @param pset - The PSET to validate
 * @returns An object with validation errors, or an empty object if valid
 */
export function validatePset(pset: Partial<Pset>): Record<string, string> {
	const errors: Record<string, string> = {};

	if (!pset.name || pset.name.trim() === '') {
		errors.name = 'Name is required';
	}

	if (pset.torque_min === undefined || pset.torque_max === undefined) {
		errors.torque = 'Torque min and max are required';
	} else if (pset.torque_min >= pset.torque_max) {
		errors.torque = 'Torque max must be greater than min';
	} else if (pset.torque_min < 0) {
		errors.torque = 'Torque values must be positive';
	}

	if (pset.angle_min === undefined || pset.angle_max === undefined) {
		errors.angle = 'Angle min and max are required';
	} else if (pset.angle_min >= pset.angle_max) {
		errors.angle = 'Angle max must be greater than min';
	} else if (pset.angle_min < 0) {
		errors.angle = 'Angle values must be positive';
	}

	return errors;
}

/**
 * Checks if a tightening result is within PSET bounds
 * @param torque - The torque value
 * @param angle - The angle value
 * @param pset - The PSET to check against
 * @returns True if within bounds, false otherwise
 */
export function isWithinPsetBounds(torque: number, angle: number, pset: Pset): boolean {
	return (
		torque >= pset.torque_min &&
		torque <= pset.torque_max &&
		angle >= pset.angle_min &&
		angle <= pset.angle_max
	);
}
