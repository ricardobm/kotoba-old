import React from 'react'
import logo from './logo.svg'
import './css/App.scss'
import { fromEvent } from 'rxjs'
import { scan, debounceTime, takeUntil, tap } from 'rxjs/operators'

import PingPong from './pages/PingPong'
import { Switch, Route } from 'react-router'
import { Link } from 'react-router-dom'
import Todos from './pages/Todos'
import Dictionary from './pages/Dictionary'

import { TiMediaEject } from 'react-icons/ti'

interface PageState {}

class App extends React.Component {
	state: PageState = {}

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
			<Switch>
				<Route exact path="/">
					<Home />
				</Route>
				<Route path="/ping_pong">
					<PingPong />
				</Route>
				<Route path="/todo" onEnter>
					<Todos />
				</Route>
				<Route path="/search/:query?">
					<Dictionary />
				</Route>
			</Switch>
		)
	}
}

const Home: React.FC = () => (
	<div className="App">
		<header className="App-header">
			<TiMediaEject />
			<img src={logo} className="App-logo" alt="logo" />
			<p>
				Edit <code>src/App.tsx</code> and save to reload.
			</p>
			<a className="App-link" href="https://reactjs.org" target="_blank" rel="noopener noreferrer">
				Learn React
			</a>
			<Link to="/todo" className="App-link">
				TODO
			</Link>
			<Link to="/search" className="App-link">
				Search
			</Link>
			<Link to="/ping_pong" className="App-link">
				Ping pong
			</Link>
		</header>
	</div>
)

export default App
