export type EventHandler<T = unknown> = (data: T) => void;

export class EventEmitter<
	T extends Record<string, unknown> = Record<string, unknown>,
> {
	private events: Map<keyof T, Set<EventHandler>> = new Map();

	/**
	 * Subscribe to an event
	 */
	on<K extends keyof T>(event: K, handler: EventHandler<T[K]>): void {
		if (!this.events.has(event)) {
			this.events.set(event, new Set());
		}
		this.events.get(event)?.add(handler);
	}

	/**
	 * Unsubscribe from an event
	 */
	off<K extends keyof T>(event: K, handler: EventHandler<T[K]>): void {
		const handlers = this.events.get(event);
		if (handlers) {
			handlers.delete(handler);
			if (handlers.size === 0) {
				this.events.delete(event);
			}
		}
	}

	/**
	 * Subscribe to an event only once
	 */
	once<K extends keyof T>(event: K, handler: EventHandler<T[K]>): void {
		const onceHandler = (data: T[K]) => {
			handler(data);
			this.off(event, onceHandler);
		};
		this.on(event, onceHandler);
	}

	/**
	 * Emit an event
	 */
	emit<K extends keyof T>(event: K, data: T[K]): void {
		const handlers = this.events.get(event);
		if (handlers) {
			handlers.forEach((handler) => {
				try {
					handler(data);
				} catch (error) {
					console.error(`Error in event handler for ${String(event)}:`, error);
				}
			});
		}
	}

	/**
	 * Remove all event listeners
	 */
	removeAllListeners(): void {
		this.events.clear();
	}
}
