import React from 'react'
import logo from './logo.svg'
import './css/App.scss'
import { Router, RouteComponentProps, Link } from '@reach/router'
import { fromEvent } from 'rxjs'
import { scan, debounceTime, takeUntil, tap } from 'rxjs/operators'

import PingPong from './pages/PingPong'

type Item = {
	id: string
	text: string
}

interface PageState extends RouteComponentProps {
	items: Item[]
	loading: boolean
	error: string
}

class App extends React.Component {
	state: PageState = {
		items: [],
		loading: true,
		error: '',
	}

	componentDidMount() {
		const click = fromEvent(document, 'click')
		const double = fromEvent(document, 'dblclick').pipe(tap(() => console.log('Double click!')))

		function handleOnClick() {
			click
				.pipe(
					debounceTime(1000),
					takeUntil(double),
					scan(count => count + 1, 0)
				)
				.subscribe({
					next: count => console.log(`Clicked ${count} times!`),
					complete: () => {
						console.log(`Completed!`)
						handleOnClick()
					},
				})
		}

		handleOnClick()

		fetch('/api/list')
			.then(data => data.json())
			.then(data => this.setState({ items: data }))
			.catch(() => this.setState({ error: 'Failed to load!' }))
			.finally(() => this.setState({ loading: false }))
	}

	render() {
		return (
			<Router>
				<Home path="/" />
				<Items items={this.state.items} loading={this.state.loading} error={this.state.error} path="/items" />
				<Dict path="/search" />
				<Dict path="/search/:search" />
			</Router>
		)
	}
}

interface DictResult {
	id: number
	expression: string
	reading: string
	total: number
	elapsed: number
	terms: DictItem[]
	tags: { [key: string]: DictTag }
	sources: { [key: string]: DictSource }
}

interface DictItem {
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

interface DictDefinition {
	text: string[]
	info: string[]
	tags: string[]
	link: DictLink[]
}

interface DictLink {
	uri: string
	title: string
}

interface DictForm {
	reading: string
	expression: string
	romaji: string
	frequency: number | null
}

interface DictState {
	search: string
	result: DictResult | null
}

interface DictTag {
	name: string
	category: string
	description: string
	order: number
	source: number
}

interface DictSource {
	name: string
	revision: string
}

interface DictProps extends RouteComponentProps {
	search?: string
	id?: string
}

class Dict extends React.Component<DictProps, DictState> {
	state = { search: this.props.search || 'Search here...', result: null }

	inputEl = React.createRef<HTMLInputElement>()

	componentDidMount() {
		this.selectAll()
		this.search(this.state.search)
	}

	selectAll() {
		if (this.inputEl.current) {
			this.inputEl.current.select()
		}
	}

	doSearch(ev: React.FormEvent<HTMLInputElement>) {
		this.search(ev.currentTarget.value)
	}

	search(text: string) {
		this.setState({ search: text })
		this.props.navigate && this.props.navigate('/search/' + text)
		fetch('/api/dict/search', {
			method: 'POST',
			body: JSON.stringify({
				query: text,
				options: {
					limit: 100,
					mode: 'Contains',
				},
			}),
		})
			.then(data => data.json())
			.then(data => {
				if (this.state.search !== text) {
					return
				}
				const sources: any = {}
				for (const it of data.sources) {
					sources[it.name] = it
				}
				data.sources = sources
				this.setState({ result: data })
			})
	}

	render() {
		return (
			<div>
				<input
					type="text"
					value={this.state.search}
					autoFocus
					ref={this.inputEl}
					onFocus={() => this.selectAll()}
					onInput={ev => this.doSearch(ev)}
				/>
				<hr />
				<div>{this.state.result ? <Result data={this.state.result!} /> : <div />}</div>
			</div>
		)
	}
}

const Result: React.FC<{ data: DictResult }> = ({ data }) => (
	<div>
		<h1>
			Results for "{data.expression}" ({data.reading})
		</h1>
		<div>
			Found {data.total} results in {data.elapsed.toFixed(3)}s
		</div>
		<hr />
		{data.terms
			.map(it => <ResultItem item={it} data={data} />)
			.reduce((acc, val): any => (acc ? [acc, <hr />, val] : [val]), null)}
	</div>
)

const ResultItem: React.FC<{ data: DictResult; item: DictItem }> = ({ data, item }) => (
	<div>
		<h2>
			<Term term={item.expression} reading={item.reading} frequency={item.frequency} />
		</h2>
		<em>
			Sources:&nbsp;
			{item.sources
				.map(src => <span title={`Revision ${data.sources[src].revision}`}>{data.sources[src].name}</span>)
				.reduce((acc, val): any => (acc ? [acc, ', ', val] : [val]), null)}
		</em>
		<div>
			<TagList data={data} tags={item.tags} />
		</div>
		<ol>
			{item.definition.map(item => (
				<ResultDefinition item={item} data={data} />
			))}
		</ol>
		{item.forms.length === 0 ? (
			<div />
		) : (
			<div>
				<h3>Other forms</h3>
				<ul>
					{item.forms.map(it => (
						<li>
							<Term term={it.expression} reading={it.reading} frequency={it.frequency} />
						</li>
					))}
				</ul>
			</div>
		)}
	</div>
)

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

const ResultDefinition: React.FC<{
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
}> = ({ tags, data, wrap }) => (
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
						.map(it => data.tags[it])
						.sort((a, b) => {
							const sA = a.order || 0
							const sB = b.order || 0
							if (sA !== sB) {
								return sA - sB
							} else {
								return a.name.toLowerCase().localeCompare(b.name.toLowerCase())
							}
						})
						.map(tag => (
							<ResultTag tag={tag} src={data.sources[tag.source].name} />
						))}
				</span>
			</span>
		)}
	</span>
)

const ResultTag: React.FC<{ tag: DictTag; src: string }> = ({ tag, src }) => (
	<span title={`${tag.description} -- ${src}`} style={{ display: 'inline-block', marginRight: '10px' }}>
		{tag.name.toLowerCase()}
	</span>
)

const LoadingMessage: React.FC<{ loading: boolean }> = ({ loading }) =>
	loading ? <div className="loading">Loading...</div> : <div />

const ErrorMessage: React.FC<{ message: string }> = ({ message }) =>
	message ? <div className="error">Error: {message}</div> : <div />

const ListItem: React.FC<{ item: Item }> = ({ item }) => <div>{item.text}</div>

const List: React.FC<{ items: Item[] }> = ({ items }) => (
	<div>
		{items.map(it => (
			<ListItem key={it.id} item={it} />
		))}
	</div>
)

const Home: React.FC<RouteComponentProps> = () => (
	<div className="App">
		<header className="App-header">
			<PingPong></PingPong>
			<img src={logo} className="App-logo" alt="logo" />
			<p>
				Edit <code>src/App.tsx</code> and save to reload.
			</p>
			<a className="App-link" href="https://reactjs.org" target="_blank" rel="noopener noreferrer">
				Learn React
			</a>
			<Link to="/items" className="App-link">
				Items
			</Link>
			<Link to="/search" className="App-link">
				Search
			</Link>
		</header>
	</div>
)

const Items: React.FC<PageState> = state => (
	<div className="App">
		<LoadingMessage loading={state.loading} />
		<ErrorMessage message={state.error} />
		<List items={state.items} />
		<Link to="/" className="App-link">
			Home
		</Link>
	</div>
)

export default App
