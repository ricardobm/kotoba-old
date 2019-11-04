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
		this.handleInput = this.handleInput.bind(this)
		this.state = {
			width: 0,
			height: 0,
		}
	}

	componentDidMount() {}

	componentWillUnmount() {}

	componentDidUpdate() {}

	render() {
		const props = this.props
		const _state = this.state
		const classes = props.classes
		const document = props.document || []

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
								<EditorBlock node={x} key={x.id} />
							))}
							<div>TOOLBAR END</div>
						</div>
					</div>
				)}
			</Measure>
		)
	}

	private handleInput(ev: React.FormEvent) {
		const txt = doc.parseDocument(ev.currentTarget)
		console.log(doc.toMarkdown(txt))
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
}

class EditorBlockBase extends React.Component<BlockProps> {
	constructor(props: BlockProps) {
		super(props)
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
					<div contentEditable suppressContentEditableWarning>
						{doc.renderInline(node.text)}
					</div>
				)
			default:
				return <pre>{doc.toMarkdown([node])}</pre>
		}
	}
}

const EditorBlock = withStyles(blockStyles)(EditorBlockBase)

export default withStyles(styles)(Editor)
