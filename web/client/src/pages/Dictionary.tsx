import React, { useEffect, useRef, useLayoutEffect } from 'react'
import { AppState } from '../store'
import { Dispatch, Action } from 'redux'
import { connect } from 'react-redux'

import { FixedSizeList as List } from 'react-window'
import AutoSizer from 'react-virtualized-auto-sizer'

import * as app from '../store/app'
import * as dictionary from '../store/dictionary'
import { DictResult, DictTerm, DictDefinition, DictTag } from '../store/dictionary'
import LinkTo from '../util/LinkTo'
import { makeStyles, createStyles } from '@material-ui/core'

interface IDispatch {
	dispatch: Dispatch<Action>
}

interface IState extends dictionary.State {}

interface IOwnProps {
	initialQuery: string
}

interface IProps extends IDispatch, IState, IOwnProps {}

const useStyles = makeStyles(() => {
	return createStyles({
		root: {
			flexGrow: 1,
			display: 'flex',
			flexDirection: 'column',
		},
		header: {
			flexShrink: 0,
		},
		results: {
			flexGrow: 1,
		},
	})
})

const Dictionary: React.FC<IProps> = self => {
	const dispatch = self.dispatch
	const doSearch = (query: string) => {
		dispatch(dictionary.search({ query }))
	}

	const inputEl = useRef<HTMLInputElement>(null)
	useEffect(() => {
		dispatch(app.setTitle(`Dictionary ${self.initialQuery && ' - ' + self.initialQuery}`))
		if (self.initialQuery !== self.query) {
			dispatch(dictionary.request({ query: self.initialQuery }))
		}

		const input = inputEl.current!
		const inputValue = input.value
		const targetValue = self.initialQuery
		if (inputValue !== targetValue) {
			setTimeout(() => {
				if (input.value === inputValue) {
					input.value = targetValue
				}
			}, 250)
		}
	}, [self.initialQuery, dispatch, self.query])

	useLayoutEffect(() => {
		setTimeout(() => inputEl.current!.select(), 100)
	}, [])

	const classes = useStyles()
	const data = self.data
	return (
		<div className={classes.root}>
			<div className={classes.header}>
				<div>
					<input
						type="text"
						ref={inputEl}
						defaultValue={self.initialQuery}
						autoFocus
						onFocus={ev => ev.currentTarget.select()}
						onChange={ev => {
							const query = ev.currentTarget.value.trim()
							doSearch(query)
						}}
					/>
					<LinkTo to="/">Home</LinkTo>
				</div>
				{data && (
					<div>
						Found {data.terms.length} results in {data.elapsed.toFixed(3)}s for {data.expression} (
						{data.reading})
					</div>
				)}
			</div>
			<div className={classes.results}>
				{data && (
					<AutoSizer disableWidth>
						{({ height }) => {
							return (
								<List width="100%" height={height} itemSize={300} itemCount={data.terms.length}>
									{({ index, style }) => {
										const it = data.terms[index]
										return (
											<div style={style} key={it.id}>
												<Entry item={it} data={data} />
											</div>
										)
									}}
								</List>
							)
						}}
					</AutoSizer>
				)}
			</div>
		</div>
	)
}

const mapStateToProps = (state: AppState, ownProps?: IOwnProps): IState => ({ ...state.dictionary, ...ownProps })

const mapDispatchToProps = (dispatch: Dispatch<Action>): IDispatch => ({
	dispatch,
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
