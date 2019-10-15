import React, { useEffect, useRef, useLayoutEffect } from 'react'
import { AppState } from '../store'
import { Dispatch, Action } from 'redux'
import { connect } from 'react-redux'
import { Link, useParams } from 'react-router-dom'

import * as dictionary from '../store/dictionary'

interface IDispatch {
	search: (query: string) => dictionary.Action
}

interface IState extends dictionary.State {}

interface IProps extends IDispatch, IState {}

const Dictionary: React.FC<IProps> = self => {
	const doSearch = (query: string) => {
		self.search(query)
	}

	const inputEl = useRef<HTMLInputElement>(null)
	const { query } = useParams()
	const urlQueryRef = useRef(query || '')
	const selfRef = useRef(self)
	useEffect(() => {
		if (urlQueryRef.current !== selfRef.current.query) {
			selfRef.current.search(urlQueryRef.current)
		}
	}, [])

	useLayoutEffect(() => {
		setTimeout(() => inputEl.current!.select(), 100)
	}, [])

	return (
		<div className="App">
			<div>
				<input
					type="text"
					ref={inputEl}
					value={self.query || ''}
					autoFocus
					onFocus={ev => ev.currentTarget.select()}
					onChange={ev => {
						const query = ev.currentTarget.value.trim()
						doSearch(query)
					}}
				/>
				<Link to="/" className="App-link">
					Home
				</Link>
			</div>
			<div>
				{self.failed && <div>Failed to load</div>}
				{self.loading && <div>Loading...</div>}
				{self.data && <div>{self.data.terms.length} results</div>}
			</div>
		</div>
	)
}

const mapStateToProps = (state: AppState): IState => ({ ...state.dictionary })

const mapDispatchToProps = (dispatch: Dispatch<Action>): IDispatch => ({
	search: (query: string) => dispatch(dictionary.search({ query })),
})

export default connect(
	mapStateToProps,
	mapDispatchToProps
)(Dictionary)
