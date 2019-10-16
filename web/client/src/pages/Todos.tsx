import { connect } from 'react-redux'
import { AppState } from '../store'
import { Action, Dispatch } from 'redux'
import { ThunkDispatch } from 'redux-thunk'
import React, { useEffect, useLayoutEffect, useRef } from 'react'

import * as app from '../store/app'
import * as todo from '../store/todo'
import LinkTo from '../util/LinkTo'

interface IState extends todo.State {}

interface IDispatch {
	request: () => Promise<todo.Action>
	add: (text: string) => todo.Action
	del: (index: number) => todo.Action
	setTitle: (title: string) => todo.Action
	dispatch: Dispatch<Action>
}

interface IProps extends IState, IDispatch {}

const Todos: React.FC<IProps> = self => {
	const inputEl = useRef<HTMLInputElement>(null)

	const handleAdd = () => {
		const elem = inputEl.current!
		const input = elem.value.trim()
		elem.value = ''
		elem.focus()
		input && self.add(input)
	}

	useEffect(() => {
		self.dispatch(app.setTitle('TODO'))
		if (!self.loaded && !self.loading) {
			self.request()
		}
	}, []) // eslint-disable-line react-hooks/exhaustive-deps

	useLayoutEffect(() => {
		const input = inputEl.current
		input && input.focus()
		setTimeout(() => input && input.focus(), 250)
	}, [])

	return (
		<div className="App">
			<LoadingMessage loading={self.loading} />
			<ErrorMessage message={self.error} />
			<List items={self.todos} onDelete={index => self.del(index)} />
			<div>
				<input
					type="text"
					ref={inputEl}
					onKeyPress={ev => {
						if (ev.which === 13 || ev.keyCode === 13) {
							handleAdd()
						}
					}}
					disabled={self.loading}
				/>
				&nbsp;
				<button onClick={handleAdd} disabled={self.loading}>
					Add
				</button>
			</div>
			<LinkTo to="/">Home</LinkTo>
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
})

const mapDispatchToProps = (dispatch: ThunkDispatch<any, any, Action>): IDispatch => ({
	request: () => dispatch(todo.request()),
	add: arg => dispatch(todo.add(arg)),
	del: arg => dispatch(todo.del(arg)),
	setTitle: arg => dispatch(todo.setTitle(arg)),
	dispatch,
})

export default connect(
	mapStateToProps,
	mapDispatchToProps
)(Todos)
