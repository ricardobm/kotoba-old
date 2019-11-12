import React from 'react'

export type DocNode = Inline | Block

export function toMarkdown(nodes: DocNode[]): string {
	const out: string[] = []
	for (const node of nodes) {
		out.push(nodeMarkdown(node, true))
	}
	return out.join('\n\n')
}

/** Base type for nodes in the document. */
export interface Base {
	id: string
	type: string
}

//============================================================================//
// Inline nodes
//============================================================================//

/** Union type for inline nodes. */
export type Inline = Text | Code | Formatted | Link | Image | LineBreak

export const TEXT = 'text'

/** Plain text node. */
export interface Text extends Base {
	type: typeof TEXT
	text: string
}

export const text = (id: string, text: string): Text => ({ id, type: TEXT, text })

export const isText = (n: Base): n is Text => n.type === TEXT

export const CODE = 'code'

/** Inline code node. */
export interface Code extends Base {
	type: typeof CODE
	text: string
}

export const code = (id: string, text: string): Code => ({ id, type: CODE, text })

export const isCode = (n: Base): n is Code => n.type === CODE

export const BR = 'br'

/** Inline line break. */
export interface LineBreak extends Base {
	type: typeof BR
	text: string
}

export const lineBreak = (id: string): LineBreak => ({ id, type: BR, text: '\n' })

export const isLineBreak = (n: Base): n is LineBreak => n.type === BR

export const DEL = 'del'
export const EM = 'em'
export const STRONG = 'strong'

/** Inline format tags. */
export type FormatTag = typeof DEL | typeof EM | typeof STRONG

/** Inline formatted text node. */
export interface Formatted extends Base {
	type: FormatTag
	text: Inline[]
}

export const formatted = (id: string, type: FormatTag, text: Inline[]): Formatted => ({ id, type, text })

export const del = (id: string, text: Inline[]) => formatted(id, DEL, text)

export const em = (id: string, text: Inline[]) => formatted(id, EM, text)

export const strong = (id: string, text: Inline[]) => formatted(id, STRONG, text)

export const isFormatTag = (v: string): v is FormatTag => [DEL, EM, STRONG].indexOf(v as FormatTag) >= 0

export const isFormatted = (n: Base): n is Formatted => isFormatTag(n.type)

export const A = 'a'

/** Link node. */
export interface Link extends Base {
	type: typeof A
	text: Inline[]
	href: string
	title?: string
}

export const link = ({
	id,
	text,
	href,
	title,
}: {
	id: string
	text: Inline[]
	href: string
	title?: string
}): Link => ({
	id,
	type: A,
	text,
	href,
	title,
})

export const isLink = (n: Base): n is Link => n.type === A

export const IMG = 'img'

/** Image node. */
export interface Image extends Base {
	type: typeof IMG
	text: Inline[]
	src: string
	title?: string
}

export const image = ({
	id,
	text,
	src: src,
	title,
}: {
	id: string
	text: Inline[]
	src: string
	title?: string
}): Image => ({
	id,
	type: IMG,
	text,
	src,
	title,
})

export const isImage = (n: Base): n is Image => n.type === IMG

//============================================================================//
// Block nodes
//============================================================================//

/** Union type for block elements. */
export type Block = Paragraph | List | Header | HBreak | Table | CodeBlock | Blockquote

export const isBlock = (n: Base): n is Block => (n as any).isBlock === true

/** Base type for block elements. */
export interface BlockBase extends Base {
	isBlock: true
}

export const H1 = 'h1'
export const H2 = 'h2'
export const H3 = 'h3'
export const H4 = 'h4'
export const H5 = 'h5'
export const H6 = 'h6'

/** Header tags. */
export type HeaderTag = typeof H1 | typeof H2 | typeof H3 | typeof H4 | typeof H5 | typeof H6

/** Header block element. */
export interface Header extends BlockBase {
	type: HeaderTag
	text: Inline[]
}

export const header = (id: string, type: HeaderTag, text: Inline[]): Header => ({ id, type, text, isBlock: true })

export const isHeaderTag = (v: string): v is HeaderTag => [H1, H2, H3, H4, H5, H6].indexOf(v as HeaderTag) >= 0

export const isHeader = (n: Base): n is Header => isHeaderTag(n.type)

export const P = 'p'

/** Paragraph block element. */
export interface Paragraph extends BlockBase {
	type: typeof P
	text: Inline[]
}

export const paragraph = (id: string, text: Inline[]): Paragraph => ({ id, type: P, text, isBlock: true })

export const isParagraph = (n: Base): n is Header => n.type === P

export const HR = 'hr'

/** Horizontal break block element. */
export interface HBreak extends BlockBase {
	type: typeof HR
}

export const hBreak = (id: string): HBreak => ({ id, type: HR, isBlock: true })

export const isHBreak = (n: Base): n is HBreak => n.type === HR

export const BLOCKQUOTE = 'blockquote'

/** Blockquote block element. */
export interface Blockquote extends BlockBase {
	type: typeof BLOCKQUOTE
	text: Block[]
}

export const blockquote = (id: string, text: Block[]): Blockquote => ({ id, type: BLOCKQUOTE, text, isBlock: true })

export const isBlockquote = (n: Base): n is Blockquote => n.type === BLOCKQUOTE

export const UL = 'ul'

/** Unordered list block element. */
export interface UList extends BlockBase {
	type: typeof UL
	ordered: false
	items: ListItem[]
}

export const uList = (id: string, items: ListItem[]): UList => ({
	id,
	type: UL,
	isBlock: true,
	ordered: false,
	items,
})

export const isUList = (n: Base): n is UList => n.type === UL

export const OL = 'ol'

/** Ordered list block element. */
export interface OList extends BlockBase {
	type: typeof OL
	ordered: true
	start: number
	items: ListItem[]
}

export const oList = (id: string, items: ListItem[], start?: number): OList => ({
	id,
	type: OL,
	isBlock: true,
	ordered: true,
	items,
	start: start && start > 0 ? start : 1,
})

export const isOList = (n: Base): n is OList => n.type === OL

/** Ordered or unordered list block element. */
export type List = OList | UList

export const isList = (n: Base): n is List => isUList(n) || isOList(n)

export const LI = 'li'

/** List item. */
export interface ListItem extends BlockBase {
	type: typeof LI
	text: Block[]
}

export const listItem = (id: string, text: Block[]): ListItem => ({ id, type: LI, isBlock: true, text })

export const isListItem = (n: Base): n is ListItem => n.type === LI

export const TABLE = 'table'

/** Table block element. */
export interface Table extends BlockBase {
	type: typeof TABLE
	head: TableRow | null
	body: TableRow[]
}

export const table = (id: string, head: TableRow | null, body: TableRow[]): Table => ({
	id,
	type: TABLE,
	isBlock: true,
	head,
	body,
})

export const isTable = (n: Base): n is Table => n.type === TABLE

export const TR = 'tr'

/** Table row. */
export interface TableRow extends BlockBase {
	type: typeof TR
	data: TableCell[]
}

export const tableRow = (id: string, data: TableCell[]): TableRow => ({ id, type: TR, isBlock: true, data })

export const isTableRow = (n: Base): n is TableRow => n.type === TR

export const TD = 'td'
export const TH = 'th'

/** Table cell. */
export interface TableCell extends BlockBase {
	type: typeof TD | typeof TH
	head: boolean
	text: Inline[]
	align?: string
}

export const tableCell = (id: string, head: boolean, text: Inline[], align?: string): TableCell => ({
	id,
	type: head ? TH : TD,
	isBlock: true,
	head,
	text,
	align,
})

export const isTableCell = (n: Base): n is TableCell => n.type === TD || n.type === TH

export const PRE = 'pre'

/** Code block element. */
export interface CodeBlock extends BlockBase {
	type: typeof PRE
	text: string
	lang: string
	info: string
}

export const codeBlock = (id: string, text: string, lang: string, info: string): CodeBlock => ({
	id,
	type: PRE,
	isBlock: true,
	text,
	lang,
	info,
})

export const isCodeBlock = (n: Base): n is CodeBlock => n.type === PRE

//============================================================================//
// HTML parsing
//============================================================================//

class ParsingContext {
	private _ids: { [key: string]: number } = {}
	private _parent?: ParsingContext
	private _rootId?: string
	private _selection: Selection | null

	anchorNode?: Base
	focusNode?: Base
	anchorOffset?: number
	focusOffset?: number

	constructor(parent?: ParsingContext, rootId?: string) {
		this._parent = parent
		this._rootId = rootId
		this._selection = (parent && parent._selection) || document.getSelection()
	}

	sel() {
		return this._selection
	}

	setSelection({ anchor, focus, offset }: { anchor?: Base; focus?: Base; offset: number }) {
		if (this._parent) {
			this._parent.setSelection({ anchor, focus, offset })
		} else {
			if (focus && anchor) {
				throw new Error('cannot specify anchor and focus node at the same time')
			}
			if (anchor) {
				this.anchorNode = anchor
				this.anchorOffset = offset
			} else if (focus) {
				this.focusNode = focus
				this.focusOffset = offset
			}
		}
	}

	sub(id: string) {
		return new ParsingContext(this, id)
	}

	id(base: string, node: Node | null): string {
		if (this._parent) {
			return this._parent.id(this._rootId ? this._rootId + '_' + base : base, node)
		}

		if (node && node.nodeType === Node.ELEMENT_NODE) {
			const elem = node as Element
			if (elem.id) {
				if (new RegExp(`^${base}:\d+$`).test(elem.id)) {
					return elem.id
				}
			}
		}

		base = base || 'id'
		const counter = (this._ids[base] || 0) + 1
		const id = `${base}:${counter}`
		this._ids[base] = counter
		return id
	}
}

export interface ParsedDocument {
	content: Block[]
	selection?: DocumentSelection
}

export interface DocumentSelection {
	anchorNode: Base
	anchorOffset: number
	focusNode: Base
	focusOffset: number
}

/**
 * Parse the child nodes of the given root element as block elements in a
 * document.
 */
export function parseDocument(root: Node, ctx?: ParsingContext): ParsedDocument {
	ctx = ctx || new ParsingContext()

	const out: Block[] = []
	let inlines: Inline[] = []
	for (const node of root.childNodes) {
		for (const it of parseElem(node, ctx)) {
			if (isBlock(it)) {
				if (inlines.length) {
					out.push(paragraph(ctx.id(P, null), inlines))
					inlines = []
				}
				out.push(it)
			} else {
				inlines.push(it as Inline)
			}
		}
	}
	if (inlines.length) {
		out.push(paragraph(ctx.id(P, null), inlines))
	}
	const anchorNode = ctx.anchorNode
	const focusNode = ctx.focusNode
	return {
		content: out,
		selection: anchorNode &&
			focusNode && {
				anchorNode,
				focusNode,
				anchorOffset: ctx.anchorOffset!,
				focusOffset: ctx.focusOffset!,
			},
	}
}

function parseElem(node: Node, ctx: ParsingContext): Base[] {
	const out: Base[] = []

	if (node.nodeType === Node.TEXT_NODE) {
		if (node.textContent) {
			const textNode = text(ctx.id(TEXT, node), node.textContent)
			pushText(out, textNode, node, ctx)
		}
	} else if (node.nodeType === Node.ELEMENT_NODE) {
		const elem = node as Element
		const type = elem.tagName.toLowerCase()
		switch (type) {
			case BLOCKQUOTE: {
				const id = ctx.id(BLOCKQUOTE, node)
				out.push(blockquote(id, parseDocument(node, ctx.sub(id)).content))
				break
			}

			case 'dd':
			case 'dl':
			case 'dt':
				break

			case 'fieldset':
			case 'form':
				// Ignore form elements
				break

			case H1:
			case H2:
			case H3:
			case H4:
			case H5:
			case H6: {
				const id = ctx.id('h', node)
				out.push(header(id, type, parseInlineContent(node, ctx.sub(id))))
				break
			}

			case HR:
				out.push(hBreak(ctx.id(HR, node)))
				break

			case PRE: {
				let lang = ''
				const info = elem.getAttribute('data-info') || ''
				for (const cls of elem.classList) {
					if (/^language-\w+$/.test(cls)) {
						lang = cls.replace(/^language-/, '')
						break
					}
				}
				out.push(codeBlock(ctx.id(PRE, node), elem.textContent || '', lang, info))
				break
			}

			case TABLE:
				out.push(parseTable(elem, ctx))
				break

			case OL: {
				const id = ctx.id(OL, node)
				const start = parseInt(elem.getAttribute('start') || '1', 10)
				out.push(oList(id, parseList(elem, ctx.sub(id)), start))
				break
			}

			case UL: {
				const id = ctx.id(UL, node)
				out.push(uList(id, parseList(elem, ctx.sub(id))))
				break
			}

			default:
				if (isParagraphElem(type)) {
					// Parse the contents of the element as blocks. If the element
					// has just inline content, it will be returned wrapped as a
					// paragraph.
					out.push(...parseDocument(elem, ctx).content)
				} else {
					const [content] = parseInlineElem(elem, ctx)
					out.push(...content)
				}
				// Just ignore unrecognized elements.
				break
		}
	}
	return out
}

/** Check if the tag name is a supported block level container. */
function isParagraphElem(tagName: string) {
	switch (tagName.toLowerCase()) {
		case 'address':
		case 'article':
		case 'aside':
		case 'details':
		case 'dialog':
		case 'div':
		case 'figure':
		case 'figcaption':
		case 'footer':
		case 'header':
		case 'hgroup':
		case 'main':
		case 'nav':
		case 'p':
		case 'section':
			return true
		default:
			return false
	}
}

function parseInlineContent(root: Node, ctx: ParsingContext): Inline[] {
	const out: Inline[] = []

	// Parse a root element with just textual content as a text node
	if (root.nodeType === Node.ELEMENT_NODE) {
		const elem = root as Element
		if (elem.children.length === 0) {
			if (elem.textContent) {
				const textNode = text(ctx.id(TEXT, elem), elem.textContent)
				pushText(out, textNode, elem, ctx)
			}
			return out
		}
	}

	// Parse the content
	let shouldBreak = false
	for (const it of root.childNodes) {
		const [content, forceLineBreak] = parseInlineElem(it, ctx)
		if (content.length) {
			if (shouldBreak) {
				out.push(lineBreak(ctx.id(BR, null)))
			}
			out.push(...content)
			shouldBreak = forceLineBreak
		}
	}
	return out
}

function pushText(out: Base[], text: Text, node: Node, ctx: ParsingContext) {
	const sel = ctx.sel()
	const txtNode = node.childNodes.length === 1 ? node.childNodes[0] : node
	if (sel && txtNode === sel.anchorNode) {
		ctx.setSelection({ anchor: text, offset: sel.anchorOffset })
	}
	if (sel && txtNode === sel.focusNode) {
		ctx.setSelection({ focus: text, offset: sel.focusOffset })
	}
	out.push(text)
}

function parseInlineElem(root: Node, ctx: ParsingContext): [Inline[], boolean] {
	const out: Inline[] = []
	let forceLineBreak = false
	if (root.nodeType === Node.TEXT_NODE) {
		if (root.textContent) {
			const textNode = text(ctx.id(TEXT, root), root.textContent)
			root.textContent && pushText(out, textNode, root, ctx)
		}
	} else if (root.nodeType === Node.ELEMENT_NODE) {
		const elem = root as Element
		const type = elem.tagName.toLowerCase()
		switch (type) {
			// Elements that we care just about the content:
			case 'abbr':
			case 'acronym':
			case 'bdi':
			case 'bdo':
			case 'big':
			case 'cite':
			case 'data':
			case 'ins':
			case 'mark':
			case 'meter':
			case 'label':
			case 'output':
			case 'progress':
			case 'small':
			case 'span':
			case 'sub':
			case 'sup':
			case 'time':
				out.push(...parseInlineContent(elem, ctx))
				break

			case A: {
				const id = ctx.id(A, elem)
				const href = elem.getAttribute('href') || ''
				const title = elem.getAttribute('title') || undefined
				const text = parseInlineContent(elem, ctx.sub(id))
				out.push(link({ id, text, href, title }))
				break
			}

			case 'audio':
				// TODO: support audio
				break

			case 'b':
			case STRONG: {
				const id = ctx.id(type, elem)
				out.push(formatted(id, STRONG, parseInlineContent(elem, ctx.sub(id))))
				break
			}

			case BR:
				out.push(lineBreak(ctx.id(BR, elem)))
				break

			case 'kbd':
			case 'samp':
			case 'tt':
			case CODE:
				if (elem.textContent) {
					const id = ctx.id(CODE, elem)
					out.push(code(id, elem.textContent))
				}
				break

			case 's':
			case DEL: {
				const id = ctx.id(DEL, elem)
				out.push(formatted(id, DEL, parseInlineContent(elem, ctx.sub(id))))
				break
			}

			case 'dfn':
			case 'i':
			case 'q':
			case 'var':
			case 'u':
			case EM: {
				// TODO: support underline
				const id = ctx.id(EM, elem)
				out.push(formatted(id, EM, parseInlineContent(elem, ctx.sub(id))))
				break
			}

			case 'picture': {
				const img = elem.querySelector(IMG)
				if (img) {
					const [content] = parseInlineElem(img, ctx)
					out.push(...content)
				}
				break
			}

			case IMG: {
				const id = ctx.id(IMG, elem)
				const src = elem.getAttribute('src') || ''
				const title = elem.getAttribute('title') || undefined
				const alt = elem.getAttribute('alt') || ''
				out.push(image({ id, src, title, text: alt ? [text(ctx.id(TEXT, null), alt)] : [] }))
				break
			}

			case 'ruby':
				// TODO: support ruby text
				break

			case 'video':
				// TODO: support video
				break

			case 'wbr': {
				const id = ctx.id(TEXT, elem)
				out.push(text(id, '\u200B'))
				break
			}

			default:
				// Handle block elements as inline elements with a forced line
				// break, ignore any other unknown elements.
				if (isParagraphElem(elem.tagName)) {
					out.push(...parseInlineContent(elem, ctx))
					forceLineBreak = true
				}
				break
		}
	}
	return [out, forceLineBreak]
}

function parseList(root: Element, ctx: ParsingContext): ListItem[] {
	const out: ListItem[] = []
	for (const el of root.children) {
		const id = ctx.id(LI, el)
		out.push(listItem(id, parseDocument(el, ctx.sub(id)).content))
	}
	return out
}

function parseTable(root: Element, ctx: ParsingContext): Table {
	const id = ctx.id(TABLE, root)
	const sub = ctx.sub(id)

	const head = root.querySelector(':scope > thead')
	const body = root.querySelector(':scope > tbody')
	const foot = root.querySelector(':scope > foot')
	const rows = getRows(root)

	const headRows = head ? parseRows(getRows(head), sub) : []
	const bodyRows = body ? parseRows(getRows(body), sub) : []
	const footRows = foot ? parseRows(getRows(foot), sub) : []
	bodyRows.push(...parseRows(rows, sub))

	const headRow = headRows.length
		? headRows.shift()
		: bodyRows.length && bodyRows[0].data.every(c => c.head)
		? bodyRows.shift()
		: undefined
	const allRows = headRows.concat(bodyRows, footRows)

	return table(id, headRow || null, allRows)

	function getRows(el: Element) {
		return el.querySelectorAll(':scope > tr')
	}
}

function parseRows(rows: NodeListOf<Element>, ctx: ParsingContext): TableRow[] {
	const out: TableRow[] = []
	for (const row of rows) {
		const rowId = ctx.id(TR, row)
		const rowCtx = ctx.sub(rowId)
		const cells = []
		for (const cell of row.querySelectorAll(':scope > th, :scope > td')) {
			const cellId = rowCtx.id(TD, cell)
			const cellCtx = rowCtx.sub(cellId)
			const head = cell.tagName.toLowerCase() === TH
			const align = cell.getAttribute('align') || undefined
			cells.push(tableCell(cellId, head, parseInlineContent(cell, cellCtx), align))
		}
		if (cells.length) {
			out.push(tableRow(rowId, cells))
		}
	}
	return out
}

//============================================================================//
// Rendering
//============================================================================//

export function renderInline(nodes: Inline[]) {
	return nodes.map(x => <InlineNode key={x.id} node={x} />)
}

const InlineNode: React.FC<{ node: Inline }> = ({ node }) => {
	switch (node.type) {
		case TEXT:
			return <span id={node.id}>{node.text}</span>
		case EM:
			return <em id={node.id}>{renderInline(node.text)}</em>
		case STRONG:
			return <strong id={node.id}>{renderInline(node.text)}</strong>
		case CODE:
			return <code id={node.id}>{node.text}</code>
		default:
			return (
				<em>
					(<b>{node.type}</b> ~ <code>{toMarkdown([node])}</code>)
				</em>
			)
	}
}

//============================================================================//
// Markdown
//============================================================================//

type typeFilter = (node: DocNode) => string | undefined

function inlineMarkdown(nodes: Inline[], start?: boolean, filter?: typeFilter): string {
	const out: string[] = []
	for (const it of nodes) {
		out.push(nodeMarkdown(it, start, filter))
		start = false
	}
	return out.join('')
}

function blockMarkdown(nodes: Block[]): string {
	const out: string[] = []
	for (const it of nodes) {
		out.push(nodeMarkdown(it, true))
	}
	return out.join('\n\n')
}

function nodeMarkdown(node: DocNode, start?: boolean, filter?: typeFilter): string {
	const escapeLinkURL = (addr: string) => addr.replace(/\\<>/g, '\\$&')
	const linkTitle = (title?: string) => (title ? ` "${title.replace(/\\"/, '\\$&')}"` : '')

	const noLineBreak = (node: DocNode) => {
		if (node.type === BR) {
			return ' '
		}
		return filter && filter(node)
	}

	const header = (node: Header, prefix: string) => {
		return `${prefix} ${inlineMarkdown(node.text, false, noLineBreak)}`
	}

	if (filter) {
		const repl = filter(node)
		if (repl != null) {
			return repl
		}
	}

	switch (node.type) {
		//====================================================================//
		// Inline elements
		//====================================================================//
		case TEXT:
			return escapeMarkdown(node.text, start)
		case CODE:
			let d = '`'
			while (node.text.indexOf(d) >= 0) {
				d += '`'
			}
			return `${d}${node.text}${d}`
		case BR:
			return '\\\n'
		case DEL:
			return `~~${inlineMarkdown(node.text, false, filter)}~~`
		case STRONG:
			return `**${inlineMarkdown(node.text, false, filter)}**`
		case EM:
			return `_${inlineMarkdown(node.text, false, filter)}_`
		case A: {
			const noLinks = (node: DocNode) => {
				if (node.type === A) {
					return inlineMarkdown(node.text, false, noLinks)
				}
				return filter && filter(node)
			}
			const href = escapeLinkURL(node.href)
			return `[${inlineMarkdown(node.text, false, noLinks)}](<${href}>${linkTitle(node.title)})`
		}
		case IMG: {
			const src = escapeLinkURL(node.src)
			return `![${inlineMarkdown(node.text, false, filter)}](<${src}>${linkTitle(node.title)})`
		}

		//====================================================================//
		// Block elements
		//====================================================================//
		case P:
			return inlineMarkdown(node.text, true, filter)

		case H1:
			return header(node, '#')
		case H2:
			return header(node, '##')
		case H3:
			return header(node, '###')
		case H4:
			return header(node, '####')
		case H5:
			return header(node, '#####')
		case H6:
			return header(node, '######')

		case PRE: {
			const info = []
			node.lang && info.push(node.lang)
			node.info && info.push(node.info)
			let d = '~~~'
			while (node.text.indexOf(d) >= 0) {
				d += '~'
			}

			const i = info.length ? ` ${info.join(' ')}` : ``
			return `${d}${i}\n${escapeHTML(node.text)}\n${d}`
		}

		case HR:
			return '=============='

		case TABLE: {
			const out: string[] = []
			const row = (row: TableRow, delim?: boolean) => {
				const out: string[] = []
				for (const cell of row.data) {
					if (delim) {
						const l = cell.align === 'center' || cell.align === 'left'
						const r = cell.align === 'center' || cell.align === 'right'
						out.push(`${l ? ':' : ''}---${r ? ':' : ''}`)
					} else {
						out.push(inlineMarkdown(cell.text, false, noLineBreak))
					}
				}
				return `| ${out.join(' | ')} |`
			}
			if (node.head) {
				out.push(row(node.head))
			}
			out.push(row(node.head || node.body[0], true))
			for (const it of node.body) {
				out.push(row(it))
			}

			return out.join('\n')
		}

		case UL: {
			const out: string[] = []
			for (const it of node.items) {
				const txt = blockMarkdown(it.text)
				out.push(indent(txt, '- ', '  '))
			}
			return out.join('\n\n')
		}

		case OL: {
			let index = node.start
			const out: string[] = []
			for (const it of node.items) {
				const pre = `${index}. `
				const ind = ' '.repeat(pre.length)
				const txt = blockMarkdown(it.text)
				out.push(indent(txt, pre, ind))
				index++
			}
			return out.join('\n\n')
		}

		case BLOCKQUOTE: {
			const txt = blockMarkdown(node.text)
			return indent(txt, '> ')
		}

		default:
			console.error(`Unhandled markdown node type: ${(node as Base).type}`, node)
			return ''
	}
}

export function escapeMarkdown(text: string, start?: boolean): string {
	// At the start of the string we escape anything that could begin a markdown
	// block (lists, breaks, titles, code blocks, tables, blockquotes)
	const escapeStart = '^\\s*([-#=~]|\\d+\\.)'
	// Escape special inline chars and HTML entities. We also escape a `!` at
	// the end because it could become an image link.
	const escapeChars = '[\\\\|~`*_\\[\\]<>&]|!$'
	const escapeRegex = start ? `${escapeStart}|${escapeChars}` : escapeChars
	return text.replace(new RegExp(escapeRegex, 'g'), s => {
		switch (s) {
			case '<':
				return '&lt;'
			case '>':
				return '&gt;'
			case '&':
				return '&amp;'
			default:
				return s.replace(/^\s*/, '$&\\')
		}
	})
}

function escapeHTML(text: string): string {
	const escapes: any = {
		'<': '&lt;',
		'>': '&gt;',
		'&': '&amp;',
	}
	return text.replace(/[<>&]/g, s => escapes[s] || s)
}

function indent(text: string, prefix: string, indent?: string) {
	const lines = text.split(/\n|\r\n?/g)
	indent = indent || prefix

	const first = `${prefix}${lines.shift()}`
	return lines.length ? first + '\n' + lines.map(x => `${indent}${x}`).join('\n') : first
}
