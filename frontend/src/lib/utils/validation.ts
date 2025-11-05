/**
 * Form validation utilities
 */

/**
 * Validates that a value is not empty
 * @param value - The value to validate
 * @param fieldName - The name of the field for error message
 * @returns Error message if invalid, empty string if valid
 */
export function validateRequired(value: unknown, fieldName: string): string {
	if (value === undefined || value === null || value === '') {
		return `${fieldName} is required`;
	}
	return '';
}

/**
 * Validates that a number is within a range
 * @param value - The number to validate
 * @param min - Minimum value (inclusive)
 * @param max - Maximum value (inclusive)
 * @param fieldName - The name of the field for error message
 * @returns Error message if invalid, empty string if valid
 */
export function validateRange(
	value: number,
	min: number,
	max: number,
	fieldName: string
): string {
	if (isNaN(value)) {
		return `${fieldName} must be a number`;
	}
	if (value < min || value > max) {
		return `${fieldName} must be between ${min} and ${max}`;
	}
	return '';
}

/**
 * Validates that a min/max pair is valid
 * @param min - The minimum value
 * @param max - The maximum value
 * @param fieldName - The name of the field for error message
 * @returns Error message if invalid, empty string if valid
 */
export function validateMinMax(min: number, max: number, fieldName: string): string {
	if (isNaN(min) || isNaN(max)) {
		return `${fieldName} values must be numbers`;
	}
	if (min >= max) {
		return `${fieldName} maximum must be greater than minimum`;
	}
	if (min < 0 || max < 0) {
		return `${fieldName} values must be positive`;
	}
	return '';
}

/**
 * Validates that a string has a minimum length
 * @param value - The string to validate
 * @param minLength - Minimum required length
 * @param fieldName - The name of the field for error message
 * @returns Error message if invalid, empty string if valid
 */
export function validateMinLength(value: string, minLength: number, fieldName: string): string {
	if (value.trim().length < minLength) {
		return `${fieldName} must be at least ${minLength} characters`;
	}
	return '';
}

/**
 * Validates that a string has a maximum length
 * @param value - The string to validate
 * @param maxLength - Maximum allowed length
 * @param fieldName - The name of the field for error message
 * @returns Error message if invalid, empty string if valid
 */
export function validateMaxLength(value: string, maxLength: number, fieldName: string): string {
	if (value.length > maxLength) {
		return `${fieldName} must be at most ${maxLength} characters`;
	}
	return '';
}

/**
 * Validates a VIN (Vehicle Identification Number)
 * @param vin - The VIN to validate
 * @returns Error message if invalid, empty string if valid
 */
export function validateVIN(vin: string): string {
	// VINs are typically 17 characters
	if (vin.length === 0) return '';
	if (vin.length !== 17) {
		return 'VIN must be 17 characters';
	}
	// VINs don't contain I, O, or Q
	if (/[IOQ]/i.test(vin)) {
		return 'VIN cannot contain I, O, or Q';
	}
	return '';
}

/**
 * Combines multiple validation results
 * @param validators - Array of validation functions that return error strings
 * @returns First error message found, or empty string if all valid
 */
export function combineValidators(...validators: (() => string)[]): string {
	for (const validator of validators) {
		const error = validator();
		if (error) return error;
	}
	return '';
}

/**
 * Validates that a percentage is between 0 and 100
 * @param value - The percentage to validate
 * @param fieldName - The name of the field for error message
 * @returns Error message if invalid, empty string if valid
 */
export function validatePercentage(value: number, fieldName: string): string {
	return validateRange(value, 0, 100, fieldName);
}

/**
 * Validates that an ID is positive
 * @param id - The ID to validate
 * @param fieldName - The name of the field for error message
 * @returns Error message if invalid, empty string if valid
 */
export function validateId(id: number, fieldName: string): string {
	if (isNaN(id) || id < 1) {
		return `${fieldName} must be a positive number`;
	}
	return '';
}
