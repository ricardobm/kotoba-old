import React from 'react'
import logo from './logo.svg'
import './App.css'

type Item = {
    id:   string
    text: string
}

class App extends React.Component {

    state: {
        items:   Item[],
        loading: boolean,
        error:   string,
    } = {
        items:   [],
        loading: true,
        error:   '',
    }

    componentDidMount() {
        fetch('/list')
            .then(data => data.json())
            .then(data => this.setState(data))
            .catch(() => this.setState({ error: 'Failed to load!'}))
            .finally(
                () => this.setState({ loading: false })
            )
    }

    render() {
        return (
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
                    <LoadingMessage loading={this.state.loading} />
                    <ErrorMessage message={this.state.error} />
                    <div>{List(this.state.items)}</div>
                </header>
            </div>
        )
    }
}

const LoadingMessage: React.FC<{ loading: boolean }> = ({ loading }) =>
    loading ? <div className="loading">Loading...</div> : <div />

const ErrorMessage: React.FC<{ message: string }> = ({ message }) =>
    message ? <div className="error">Error: {message}</div> : <div />

const ListItem = (item: Item) => (
    <div key={item.id}>{item.text}</div>
)

function List(list: Item[]) {
    return list.map(it => ListItem(it))
}

export default App
