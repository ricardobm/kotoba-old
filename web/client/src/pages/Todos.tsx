import * as todo from '../store/todo'
import { connect } from 'react-redux'
import { AppState } from '../store'
import { Action } from 'redux'
import { ThunkDispatch } from 'redux-thunk'
import React, { useEffect } from 'react'
import { Link } from 'react-router-dom'

interface IState extends todo.State {
	inputEl: React.RefObject<HTMLInputElement>
}

interface IDispatch {
	request: () => Promise<todo.Action>
	add: (text: string) => todo.Action
	del: (index: number) => todo.Action
	setTitle: (title: string) => todo.Action
}

interface IProps extends IState, IDispatch {}

const Todos: React.FC<IProps> = state => {
	const handleAdd = () => {
		const inputEl = state.inputEl.current!
		const input = inputEl.value.trim()
		inputEl.value = ''
		inputEl.focus()
		input && state.add(input)
	}

	useEffect(() => {
		if (!state.loaded && !state.loading) {
			state.request()
		}
	})

	return (
		<div className="App">
			<LoadingMessage loading={state.loading} />
			<ErrorMessage message={state.error} />
			<List items={state.todos} onDelete={index => state.del(index)} />
			<div>
				<input
					type="text"
					ref={state.inputEl}
					onKeyPress={ev => {
						if (ev.which === 13 || ev.keyCode === 13) {
							handleAdd()
						}
					}}
					disabled={state.loading}
				/>
				&nbsp;
				<button onClick={handleAdd} disabled={state.loading}>
					Add
				</button>
			</div>
			<Link to="/" className="App-link">
				Home
			</Link>
		</div>
	)
}

const LoadingMessage: React.FC<{ loading: boolean }> = ({ loading }) =>
	loading ? <div className="loading">Loading...</div> : null

const ErrorMessage: React.FC<{ message?: string }> = ({ message }) =>
	message ? <div className="error">Error: {message}</div> : <div />

type DeleteHandler = (index: number) => void

const List: React.FC<{ items: string[]; onDelete?: DeleteHandler }> = ({ items, onDelete }) => (
	<div>
		{items.map((it, idx) => (
			<div key={idx}>
				<span style={{ display: 'inline-block', width: '200px' }}>{it}</span>
				{onDelete && <button onClick={() => onDelete(idx)}>x</button>}
			</div>
		))}
	</div>
)

const mapStateToProps = (state: AppState): IState => ({
	...state.todo,
	inputEl: React.createRef<HTMLInputElement>(),
})

const mapDispatchToProps = (dispatch: ThunkDispatch<any, any, Action>): IDispatch => ({
	request: () => dispatch(todo.request()),
	add: arg => dispatch(todo.add(arg)),
	del: arg => dispatch(todo.del(arg)),
	setTitle: arg => dispatch(todo.setTitle(arg)),
})

export default connect(
	mapStateToProps,
	mapDispatchToProps
)(Todos)
