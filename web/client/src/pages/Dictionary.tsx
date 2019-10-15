import React, { useEffect, useRef, useLayoutEffect } from 'react'
import { AppState } from '../store'
import { Dispatch, Action } from 'redux'
import { connect } from 'react-redux'
import { Link, useParams } from 'react-router-dom'

import * as dictionary from '../store/dictionary'
import { DictResult, DictTerm, DictDefinition, DictTag } from '../store/dictionary'

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

	const data = self.data

	return (
		<div>
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
				{data && (
					<div>
						Found {data.terms.length} results in {data.elapsed.toFixed(3)}s for {data.expression} (
						{data.reading})
						{data.terms.map(it => (
							<Entry key={it.id} item={it} data={data} />
						))}
					</div>
				)}
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

const Entry: React.FC<{ data: DictResult; item: DictTerm }> = ({ data, item }) => {
	const getSource = (key: string) => {
		const src = data.sources[key]
		return (
			src && (
				<span key={src.name} title={`Revision ${src.revision}`}>
					{src.name}
				</span>
			)
		)
	}
	return (
		<div>
			<hr />
			<h2>
				<Term term={item.expression} reading={item.reading} frequency={item.frequency} />
			</h2>
			<em>
				Sources:&nbsp;
				{item.sources.map(getSource).reduce((acc, val): any => (acc ? [acc, ', ', val] : [val]), null)}
			</em>
			<div>
				<TagList data={data} tags={item.tags} />
			</div>
			<ol>
				{item.definition.map((item, idx) => (
					<Definition key={idx} item={item} data={data} />
				))}
			</ol>
			{item.forms.length === 0 ? (
				<div />
			) : (
				<div>
					<h3>Other forms</h3>
					<ul>
						{item.forms.map((it, idx) => (
							<li key={idx}>
								<Term term={it.expression} reading={it.reading} frequency={it.frequency} />
							</li>
						))}
					</ul>
				</div>
			)}
		</div>
	)
}

const Term: React.FC<{
	term: string
	reading: string
	frequency?: number | null
}> = ({ term, reading, frequency }) => (
	<span>
		{term}
		{reading !== term ? (
			<em
				style={{
					fontWeight: 'normal',
					fontSize: '0.8em',
					paddingLeft: '15px',
				}}
			>
				({reading})
			</em>
		) : (
			<span />
		)}
		{frequency ? (
			<em
				style={{
					fontWeight: 'normal',
					fontSize: '0.7em',
					paddingLeft: '15px',
					color: '#A0A0A0',
				}}
			>
				#{frequency}
			</em>
		) : (
			<em />
		)}
	</span>
)

const Definition: React.FC<{
	data: DictResult
	item: DictDefinition
}> = ({ data, item }) => (
	<li>
		<div>
			{item.text.join(', ')}
			{item.info.length ? <em>{item.info.join(', ')}</em> : <em />}
			<TagList tags={item.tags} data={data} />
		</div>
		{item.link.length === 0 ? (
			<div />
		) : (
			<ul>
				{item.link.map(it => (
					<li>
						<a href={it.uri}>{it.title}</a>
					</li>
				))}
			</ul>
		)}
	</li>
)

const TagList: React.FC<{
	tags: string[]
	data: DictResult
	wrap?: boolean
}> = ({ tags, data, wrap }) => {
	const getTagOutput = (tag: DictTag) => {
		const src = tag && data.sources[tag.source]
		return tag && <ResultTag key={tag.name} tag={tag} src={src && src.name} />
	}
	return (
		<span>
			{!tags.length ? (
				<span />
			) : (
				<span>
					{wrap ? <br /> : <span />}
					<span
						style={{
							display: 'inline-block',
							marginLeft: '20px',
							color: 'red',
						}}
					>
						{tags
							.map(key => data.tags[key])
							.sort((a, b) => {
								const sA = a.order || 0
								const sB = b.order || 0
								if (sA !== sB) {
									return sA - sB
								} else {
									return a.name.toLowerCase().localeCompare(b.name.toLowerCase())
								}
							})
							.map(getTagOutput)}
					</span>
				</span>
			)}
		</span>
	)
}

const ResultTag: React.FC<{ tag: DictTag; src: string }> = ({ tag, src }) => (
	<span title={`${tag.description} -- ${src}`} style={{ display: 'inline-block', marginRight: '10px' }}>
		{tag.name.toLowerCase()}
	</span>
)
