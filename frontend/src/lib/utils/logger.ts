/**
 * Logging utility with configurable log levels
 * Provides centralized logging with the ability to disable logs in production
 */

type LogLevel = 'debug' | 'info' | 'warn' | 'error';

const LOG_LEVEL = (import.meta.env.VITE_LOG_LEVEL as LogLevel) || 'info';

const logLevels: Record<LogLevel, number> = {
	debug: 0,
	info: 1,
	warn: 2,
	error: 3
};

function shouldLog(level: LogLevel): boolean {
	return logLevels[level] >= logLevels[LOG_LEVEL];
}

/**
 * Logs a debug message (only in development)
 * @param message - Message or data to log
 * @param optionalParams - Additional parameters to log
 */
export function debug(message?: any, ...optionalParams: any[]): void {
	if (shouldLog('debug')) {
		console.log('[DEBUG]', message, ...optionalParams);
	}
}

/**
 * Logs an informational message
 * @param message - Message or data to log
 * @param optionalParams - Additional parameters to log
 */
export function info(message?: any, ...optionalParams: any[]): void {
	if (shouldLog('info')) {
		console.log('[INFO]', message, ...optionalParams);
	}
}

/**
 * Logs a warning message
 * @param message - Message or data to log
 * @param optionalParams - Additional parameters to log
 */
export function warn(message?: any, ...optionalParams: any[]): void {
	if (shouldLog('warn')) {
		console.warn('[WARN]', message, ...optionalParams);
	}
}

/**
 * Logs an error message
 * @param message - Message or data to log
 * @param optionalParams - Additional parameters to log
 */
export function error(message?: any, ...optionalParams: any[]): void {
	if (shouldLog('error')) {
		console.error('[ERROR]', message, ...optionalParams);
	}
}

/**
 * Logger object with all logging methods
 */
export const logger = {
	debug,
	info,
	warn,
	error
};
