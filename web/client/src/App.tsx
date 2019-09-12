import React from 'react'
import logo from './logo.svg'
import './App.css'
import { Router, RouteComponentProps, Link } from '@reach/router'

type Item = {
    id: string
    text: string
}

interface PageState extends RouteComponentProps {
    items: Array<Item>,
    loading: boolean,
    error: string,
}

class App extends React.Component {

    state: PageState = {
        items: [],
        loading: true,
        error: '',
    }

    componentDidMount() {
        fetch('/api/list')
            .then(data => data.json())
            .then(data => this.setState({ items: data }))
            .catch(() => this.setState({ error: 'Failed to load!' }))
            .finally(
                () => this.setState({ loading: false })
            )
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
    query: string,
    reading: string,
    total: number,
    elapsed: number,
    terms: DictItem[],
    tags: { [key: number]: DictTag },
    sources: DictSource[],
}

interface DictItem {
    expression: string,
    reading: string,
    definition: DictDefinition[],
    romaji: string,
    source: number[],
    forms: DictForm[],
    tags: number[],
    frequency: number,
    score: number,
}

interface DictDefinition {
    text: string[],
    info: string[],
    tags: number[],
    link: DictLink[],
}

interface DictLink {
    uri: string,
    title: string,
}

interface DictForm {
    reading: string,
    expression: string,
    romaji: string,
}

interface DictState {
    search: string,
    result: DictResult | null,
}

interface DictTag {
    name: string,
    category: string,
    description: string,
    order: number,
    source: number,
}

interface DictSource {
    name: string,
    revision: string,
}

interface DictProps extends RouteComponentProps {
    search?: string,
    id?: string,
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
        fetch('/api/search', {
            method: 'POST',
            body: JSON.stringify({
                query: text,
                options: {
                    limit: 100,
                    mode: 'Contains',
                }
            })
        })
            .then(data => data.json())
            .then(data => {
                if (this.state.search !== text) {
                    return
                }
                this.setState({ result: data })
            })
    }

    render() {
        return (
            <div>
                <input type="text" value={this.state.search} autoFocus
                    ref={this.inputEl}
                    onFocus={() => this.selectAll()}
                    onInput={(ev) => this.doSearch(ev)}
                />
                <hr />
                <div>{this.state.result ? <Result data={this.state.result!} /> : <div />}</div>
            </div>
        )
    }
}

const Result: React.FC<{ data: DictResult }> = ({ data }) =>
    <div>
        <h1>Results for "{data.query}" ({data.reading})</h1>
        <div>Found {data.total} results in {data.elapsed.toFixed(3)}s</div>
        <hr />
        {data.terms.map(it => <ResultItem item={it} data={data} />)}
    </div>

const ResultItem: React.FC<{ data: DictResult, item: DictItem }> = ({ data, item }) =>
    <div>
        <h2>{item.expression}{item.reading !== item.expression ? <span>&nbsp;({item.reading})</span> : <span />}</h2>
        <em>
            Sources:&nbsp;
            {item.source.map(src =>
                <span title={`Revision ${data.sources[src].revision}`}>
                    {data.sources[src].name}
                </span>
            ).reduce(
                (acc, val): any => acc ? [acc, ', ', val] : [val],
                null
            )}

        </em>
        <div>
            <TagList data={data} tags={item.tags} />
        </div>
        <ol>
            {item.definition.map(item => <ResultDefinition item={item} data={data} />)}
        </ol>
        {item.forms.length === 0 ? <div /> :
            <div>
                <h3>Other forms</h3>
                {item.forms.map(it =>
                    <div>
                        {it.expression}
                        {it.reading !== it.expression ? <span>({it.reading})</span> : <span />}
                    </div>)
                }
            </div>
        }
    </div>

const ResultDefinition: React.FC<{ data: DictResult, item: DictDefinition }> = ({ data, item }) =>
    <li>
        <div>
            {item.text.join(', ')}
            {item.info.length ? <em>{item.info.join(', ')}</em> : <em />}
            <TagList tags={item.tags} data={data} />
        </div>
        {item.link.length === 0 ? <div /> :
            <ul>
                {item.link.map(it => <li><a href={it.uri}>{it.title}</a></li>)}
            </ul>
        }
    </li>

const TagList: React.FC<{ tags: number[], data: DictResult, wrap?: boolean }> = ({ tags, data, wrap }) =>
    <span>
        {!tags.length ? <span /> :
            <span>
                {wrap ? <br /> : <span />}
                <span style={{ display: 'inline-block', marginLeft: '20px', color: 'red' }}>
                    {tags.map(it => data.tags[it]).sort((a, b) => {
                        let sA = a.order || 0
                        let sB = b.order || 0
                        if (sA !== sB) {
                            return sA - sB
                        } else {
                            return a.name.toLowerCase().localeCompare(b.name.toLowerCase())
                        }
                    }).map(tag => <ResultTag tag={tag} src={data.sources[tag.source].name} />)}
                </span>
            </span>
        }
    </span>

const ResultTag: React.FC<{ tag: DictTag, src: string }> = ({ tag, src }) =>
    <span title={`${tag.description} -- ${src}`} style={{ display: 'inline-block', marginRight: '10px' }}>
        {tag.name.toLowerCase()}
    </span>

const LoadingMessage: React.FC<{ loading: boolean }> = ({ loading }) =>
    loading ? <div className="loading">Loading...</div> : <div />

const ErrorMessage: React.FC<{ message: string }> = ({ message }) =>
    message ? <div className="error">Error: {message}</div> : <div />

const ListItem: React.FC<{ item: Item }> = ({ item }) => (
    <div>{item.text}</div>
)

const List: React.FC<{ items: Item[] }> = ({ items }) => (
    <div>
        {items.map(it => <ListItem key={it.id} item={it} />)}
    </div>
)

const Home: React.FC<RouteComponentProps> = () => (
    <div className="App">
        <header className="App-header">
            <img src={logo} className="App-logo" alt="logo" />
            <p>Edit <code>src/App.tsx</code> and save to reload.</p>
            <a
                className="App-link"
                href="https://reactjs.org"
                target="_blank"
                rel="noopener noreferrer"
            >
                Learn React
            </a>
            <Link to="/items" className="App-link">Items</Link>
            <Link to="/search" className="App-link">Search</Link>
        </header>
    </div>
)

const Items: React.FC<PageState> = (state) => (
    <div className="App">
        <LoadingMessage loading={state.loading} />
        <ErrorMessage message={state.error} />
        <List items={state.items} />
        <Link to="/" className="App-link">Home</Link>
    </div>
)

export default App
