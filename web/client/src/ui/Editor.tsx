import React from 'react'
import {
	Theme,
	createStyles,
	withStyles,
	Toolbar,
	AppBar,
	IconButton,
	Button,
	Divider,
	Tooltip,
} from '@material-ui/core'
import Measure, { ContentRect } from 'react-measure'
import { WithStyles } from '@material-ui/styles'
import HideOnScroll from './HideOnScroll'

import SaveIcon from '@material-ui/icons/Save'
import CancelIcon from '@material-ui/icons/Cancel'
import UndoIcon from '@material-ui/icons/Undo'
import RedoIcon from '@material-ui/icons/Redo'

import { doc } from './edit'

const DEBUG = true

export interface EditorProps extends WithStyles<typeof styles> {
	/** CSS width for the list root element. */
	width?: string | number

	/** CSS height for the list root element. */
	height?: string | number

	/** Additional CSS styling applied to the root element. */
	style?: React.CSSProperties

	/** Document being edited. */
	document?: doc.DocNode[]
}

interface EditorState {
	width: number
	height: number
	document: doc.DocNode[]
	selection?: doc.DocumentSelection
}

const styles = (theme: Theme) => {
	return createStyles({
		root: {
			overflowY: 'auto',
		},
		toolbar: {
			backgroundColor: theme.palette.background.paper,
			'& *': {
				color: theme.palette.text.primary,
			},
		},
		editor: {
			padding: theme.spacing(2, 2, 2, 0),
		},
		spacer: {
			display: 'inline-block',
			width: theme.spacing(1),
		},
	})
}

class Editor extends React.Component<EditorProps, EditorState> {
	// Root element for the container
	rootEl: Element | null = null

	constructor(props: EditorProps) {
		super(props)
		this.handleResize = this.handleResize.bind(this)
		this.handleEditorChange = this.handleEditorChange.bind(this)
		this.state = {
			width: 0,
			height: 0,
			document: props.document || [],
		}
	}

	componentDidMount() {}

	componentWillUnmount() {}

	componentDidUpdate() {
		const sel = this.state.selection
		const getText = (id: string) => {
			let el: Node | null = document.getElementById(id)
			while (el && el.childNodes.length > 0) {
				el = el.childNodes[0]
			}
			return el
		}
		if (sel) {
			const anchor = getText(sel.anchorNode.id)
			const focus = getText(sel.focusNode.id)
			if (anchor && focus) {
				const range = document.createRange()
				range.setStart(anchor, sel.anchorOffset)
				range.setEnd(focus, sel.focusOffset)
				const selection = document.getSelection()
				if (selection) {
					selection.removeAllRanges()
					selection.addRange(range)
				}
			}
		}
	}

	render() {
		const props = this.props
		const state = this.state
		const classes = props.classes
		const document = state.document

		const style: React.CSSProperties = {
			width: '100%',
			...props.style,
		}
		if (props.width != null) {
			style.width = typeof props.width === 'string' ? props.width : `${props.width}px`
		}
		if (props.height != null) {
			style.height = typeof props.height === 'string' ? props.height : `${props.height}px`
		}

		const log = DEBUG ? console.log.bind(console, '[RENDER]') : () => {}
		return (
			<Measure client innerRef={el => (this.rootEl = el)} onResize={this.handleResize}>
				{({ measureRef }) => (
					<div ref={measureRef} className={classes.root} style={style}>
						<AppBar className={classes.toolbar} position="sticky">
							<Toolbar variant="dense">
								<Button startIcon={<SaveIcon />}>Save</Button>
								<IconButton aria-label="Cancel">
									<CancelIcon />
								</IconButton>
								<span className={classes.spacer} />
								<Tooltip title="Undo">
									<IconButton aria-label="Undo">
										<UndoIcon />
									</IconButton>
								</Tooltip>
								<Tooltip title="Redo">
									<IconButton aria-label="Redo">
										<RedoIcon />
									</IconButton>
								</Tooltip>
							</Toolbar>
						</AppBar>
						<div className={classes.editor}>
							{document.map(x => (
								<EditorBlock node={x} key={x.id} onChange={this.handleEditorChange} />
							))}
							<div>TOOLBAR END</div>
						</div>
					</div>
				)}
			</Measure>
		)
	}

	private handleEditorChange(before: doc.DocNode, after: doc.DocNode, selection?: doc.DocumentSelection) {
		const document = this.state.document
		const index = this.state.document.indexOf(before)
		document[index] = after
		this.setState({ document, selection })
	}

	private handleResize(rect: ContentRect) {
		const size = rect.client!
		this.setState(() => ({ width: size.width, height: size.height }))
	}
}

const blockStyles = (theme: Theme) => {
	return createStyles({
		block: {
			margin: theme.spacing(1, 0, 1, 0),
		},
	})
}

interface BlockProps extends WithStyles<typeof blockStyles> {
	node: doc.DocNode
	onChange?: (before: doc.DocNode, after: doc.DocNode, sel?: doc.DocumentSelection) => void
}

class EditorBlockBase extends React.Component<BlockProps> {
	constructor(props: BlockProps) {
		super(props)
		this.handleChange = this.handleChange.bind(this)
	}

	render() {
		const node = this.props.node
		const classes = this.props.classes
		return <div className={classes.block}>{this.renderNode(node)}</div>
	}

	renderNode(node: doc.DocNode) {
		switch (node.type) {
			case doc.P:
				return (
					<div
						contentEditable
						onInput={this.handleChange}
						dangerouslySetInnerHTML={{ __html: doc.renderInline(node.text) }}
					></div>
				)
			default:
				return <pre>{doc.toMarkdown([node])}</pre>
		}
	}

	handleChange(ev: React.FormEvent) {
		const parsed = doc.parseDocument(ev.currentTarget)
		console.log(parsed)
		console.log(doc.toMarkdown(parsed.content))
		if (this.props.onChange) {
			this.props.onChange(this.props.node, parsed.content[0], parsed.selection)
		}
	}
}

const EditorBlock = withStyles(blockStyles)(EditorBlockBase)

export default withStyles(styles)(Editor)
