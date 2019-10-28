const envHasBigInt64Array = typeof BigInt64Array !== 'undefined'

export function deepEqual(a: any, b: any) {
	// Adapted from https://github.com/epoberezkin/fast-deep-equal/blob/master/src/index.jst
	if (a === b) {
		return true
	}

	if (typeof a !== typeof b) {
		return false
	}

	if (a && b && typeof a === 'object') {
		if (a.constructor !== b.constructor) {
			return false
		}

		if (Array.isArray(a)) {
			if (a.length !== b.length) {
				return false
			}

			for (let i = 0; i < a.length; i++) {
				if (!deepEqual(a[i], b[i])) {
					return false
				}
			}

			return true
		}

		if (a instanceof Map) {
			if (a.size !== b.size) {
				return false
			}
			for (const it of a.entries()) {
				if (!b.has(it[0])) {
					return false
				}
			}
			for (const it of a.entries()) {
				if (!deepEqual(it[1], b.get(it[0]))) {
					return false
				}
			}
			return true
		}

		if (a instanceof Set) {
			if (a.size !== b.size) {
				return false
			}
			for (const it of a.entries()) {
				if (!b.has(it[0])) {
					return false
				}
			}
			return true
		}

		if (
			a.constructor.BYTES_PER_ELEMENT &&
			(a instanceof Int8Array ||
				a instanceof Uint8Array ||
				a instanceof Uint8ClampedArray ||
				a instanceof Int16Array ||
				a instanceof Uint16Array ||
				a instanceof Int32Array ||
				a instanceof Uint32Array ||
				a instanceof Float32Array ||
				a instanceof Float64Array ||
				(envHasBigInt64Array && (a instanceof BigInt64Array || a instanceof BigUint64Array)))
		) {
			const length = a.length
			if (length !== b.length) {
				return false
			}
			for (let i = 0; i < length; i++) {
				if (a[i] !== b[i]) {
					return false
				}
			}
			return true
		}

		if (a.constructor === RegExp) {
			return a.source === b.source && a.flags === b.flags
		}

		if (a.valueOf !== Object.prototype.valueOf) {
			return a.valueOf() === b.valueOf()
		}

		if (a.toString !== Object.prototype.toString) {
			return a.toString() === b.toString()
		}

		const keys = Object.keys(a)
		const length = keys.length
		if (length !== Object.keys(b).length) {
			return false
		}

		for (let i = 0; i < length; i++) {
			if (!Object.prototype.hasOwnProperty.call(b, keys[i])) {
				return false
			}
		}

		for (let i = 0; i < length; i++) {
			const key = keys[i]
			if (!deepEqual(a[key], b[key])) {
				return false
			}
		}

		return true
	}

	return typeof a === 'number' && isNaN(a) && isNaN(b)
}
