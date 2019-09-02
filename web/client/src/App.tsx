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
            </Router>
        )
    }
}

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
