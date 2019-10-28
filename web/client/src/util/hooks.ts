import { useRef } from 'react'

interface IUseCallbacks {
	[key: string]: (...args: any[]) => any
}

/** Prevent unnecessary component updates due to callback handling */
export const useCallbacks = <T extends IUseCallbacks>(init: () => T): T => {
	// `callbacks` contains the updated set of callbacks to call,
	// `handlers` is generated only once and calls `callbacks`
	const data = useRef({ callbacks: {}, handlers: {} })
	const callbacks = data.current.callbacks as T
	const handlers = data.current.handlers as T

	// Regenerate callbacks every time from the init return
	const initCallbacks = init()
	for (const cb in initCallbacks) {
		callbacks[cb] = initCallbacks[cb]
		// Generate the handler if necessary
		if (!handlers[cb]) {
			const h = handlers as any
			h[cb] = (...args: any) => callbacks[cb](...args)
		}
	}

	return handlers
}
