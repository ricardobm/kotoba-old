import { request, RequestOptions } from './ajax'
import { map, catchError } from 'rxjs/operators'

export interface QueryOptions extends RequestOptions {
	query: string
	variables?: { [key: string]: any } | null
}

interface GraphQLRoot {
	data?: QueryData
	errors?: GraphQLError[]
}

export interface QueryData {
	[key: string]: any
}

export interface GraphQLError {
	message: string
	locations: Array<{ line: number; column: number }>
	extensions?: { [key: string]: any }
}

export class QueryError extends Error {
	constructor(errors: GraphQLError[]) {
		console.error(
			'[QUERY]',
			errors
				.map(e => {
					const loc = e.locations.map(x => `L${x.line},${x.column}`).join(' ')
					return `${e.message}${loc ? ' @ ' + loc : ''}`
				})
				.join(' / ')
		)
		super('query failed: ' + errors.map(e => e.message).join(', '))
	}
}

export function query(options: QueryOptions) {
	const requestOptions: QueryOptions = {
		...options,
		url: '/api/graphql',
		method: 'POST',
		headers: {
			'Content-Type': 'application/json',
		},
		body: {
			query: options.query,
			variables: options.variables || null,
			operationName: null,
		},
	}
	delete requestOptions.query
	delete requestOptions.variables

	return request(requestOptions).pipe(
		map(data => {
			const root = data.response as GraphQLRoot
			if (root.errors && root.errors.length) {
				throw new QueryError(root.errors)
			}
			return root.data!
		}),
		catchError(err => {
			if (err.status === 400 && err.response && err.response.errors) {
				throw new QueryError(err.response.errors)
			}
			throw err
		})
	)
}
