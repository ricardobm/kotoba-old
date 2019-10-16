import React from 'react'
import logo from './logo.svg'
import './css/App.scss'

import PingPong from './pages/PingPong'
import { Switch, Route } from 'react-router'
import { Link } from 'react-router-dom'
import Todos from './pages/Todos'
import Dictionary from './pages/Dictionary'

import { TiMediaEject } from 'react-icons/ti'

class App extends React.Component {
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
