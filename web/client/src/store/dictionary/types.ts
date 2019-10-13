export enum DictActionTypes {
	QUERY_FETCH_REQUEST = '@dict:query/fetch_request',
	QUERY_FETCH_SUCCESS = '@dict:query/fetch_success',
	QUERY_FETCH_FAILURE = '@dict:query/fetch_failure',
}

export interface DictState {
	readonly loading: boolean
	readonly data: DictResult
	readonly query: DictQuery
}

/**
 * Arguments for a dictionary query.
 */
export interface DictQuery {
	query: string
}

/**
 * Root result for a dictionary query.
 */
export interface DictResult {
	id: number
	expression: string
	reading: string
	total: number
	elapsed: number
	terms: DictTerm[]
	tags: { [key: string]: DictTag }
	sources: { [key: string]: DictSource }
}

/**
 * Single term from a dictionary query.
 */
export interface DictTerm {
	expression: string
	reading: string
	definition: DictDefinition[]
	romaji: string
	sources: string[]
	forms: DictForm[]
	tags: string[]
	frequency: number | null
	score: number
}

/**
 * English definition for a `DictTerm`.
 */
export interface DictDefinition {
	text: string[]
	info: string[]
	tags: string[]
	link: DictLink[]
}

/**
 * Reference URL for a `DictDefinition`.
 */
export interface DictLink {
	uri: string
	title: string
}

/**
 * Additional forms for a `DictItem`.
 */
export interface DictForm {
	reading: string
	expression: string
	romaji: string
	frequency: number | null
}

/**
 * Tag definition for a dictionary.
 */
export interface DictTag {
	name: string
	category: string
	description: string
	order: number
	source: number
}

/**
 * Source information for a dictionary.
 */
export interface DictSource {
	name: string
	revision: string
}
