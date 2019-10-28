import React from 'react'
import { Theme, createStyles, withStyles } from '@material-ui/core'
import Measure, { ContentRect } from 'react-measure'

import { WithStyles } from '@material-ui/styles'
import { deepEqual } from '../util/common'

/**
 * Default minimum overscan height for the `VirtualScroller`.
 */
const DEFAULT_OVERSCAN_MINIMUM = 1000

/**
 * Default number of pages to overscan. Can be a decimal number.
 */
const DEFAULT_OVERSCAN_PAGES = 3

/**
 * Default first approximation for an item average.
 */
const DEFAULT_ESTIMATED_ITEM_HEIGHT = 500

/**
 * Cooldown period after a user scroll event in which the scroll position
 * won't be touched to adjust scrolling errors.
 */
const USER_SCROLL_CORRECTION_COOLDOWN_MS = 100

/**
 * Don't fix scrolling if the difference between the target position and actual
 * position is this or less.
 */
const ALLOWED_SCROLL_ERROR_PX = 10

/** Name for the attribute that stores the virtual row index. */
const DATA_INDEX_KEY = 'data-index'

export interface VirtualScrollerProps<T = any> extends WithStyles<typeof styles> {
	/**
	 * Data to render. This is provided to the render function.
	 *
	 * Besides being provided as argument to the render function, the only
	 * other function for this is to cause a re-render in case the data
	 * changes.
	 */
	data: T

	/** Number of items in the list. */
	itemCount: number

	/**
	 * Render function. This is called to provide rendering for individual items
	 * as they are scrolled into view.
	 */
	render: (index: number, data: T) => React.ReactNode

	/** CSS width for the list root element. */
	width?: string | number

	/** CSS height for the list root element. */
	height?: string | number

	/** Additional CSS styling applied to the root element. */
	style?: React.CSSProperties

	/** Minimum number of pixels to overscan, regardless of page size. */
	overscanMinimun?: number

	/** Number of pages (based on the client height) to overscan. */
	overscanPages?: number

	/**
	 * Initial estimate for the height of an item.
	 *
	 * This mostly affects the number of items in the initial rendering for
	 * the list and is quickly replaced by an estimate based on the actual
	 * item heights.
	 */
	estimatedItemHeight?: number
}

/** Represents an anchor position for scrolling. */
interface ScrollAnchor {
	/** Index of the item under the scroll position. */
	index: number
	/** Offset of the scroll position within the item. */
	offset: number
	/** If true, anchors to the end of the element/container */
	end: boolean
}

/** Layout information for the list. */
interface Layout {
	/** First index to render, inclusive. */
	minIndex: number
	/** Index after the last index to render. */
	maxIndex: number
	/** Offset of `minIndex` from the start of the list. */
	minIndexOffset: number
	/** Scroll position for this layout. */
	scroll: number
	/** Total list height (estimated). */
	totalHeight: number
}

/** Information used for layout information. */
interface LayoutInfo {
	/** Number of items to render. */
	itemCount: number
	/** Available client width. */
	width: number
	/** Available client height. */
	height: number
	/** Scroll position. */
	scroll: number
	/** Desired scroll target. */
	anchor?: ScrollAnchor
	/**
	 * Render ID incremented each time the component renders.
	 *
	 * We use this to force the layout to recalculate each time rendering
	 * occurs. This causes a cycle of render -> layout -> render that will
	 * repeat until a stable state is reached.
	 *
	 * The reason we need this is that our layout is calculated with estimated
	 * heights that are refined by measuring the actual heights of rendered
	 * rows.
	 */
	renderID: number
}

interface State {
	/** Component client width (from `react-measure`) */
	width: number
	/** Component client height (from `react-measure`) */
	height: number
	/** Computed layout for the component. */
	layout: Layout
}

const styles = (theme: Theme) => {
	return createStyles({
		root: {
			overflowY: 'auto',
			position: 'relative',
			outline: 0,
		},
		body: {
			position: 'absolute',
			width: '100%',
		},
		head: {
			position: 'absolute',
			width: 1,
		},
		row: {
			padding: '1px 0 1px 0',
		},
	})
}

const DEBUG = true
const DEBUG_TIME = false

/**
 * Provides virtual vertical scrolling for a list of items.
 *
 * This scroller does not require that the user provide item heights, relying
 * instead on actual measures of the rendered items and estimating the height
 * for unknown items.
 */
class VirtualScroller extends React.Component<VirtualScrollerProps, State> {
	/**
	 * Root element for the component. This is the scrollable container.
	 */
	rootEl: Element | null = null

	/**
	 * The head is an invisible element that is used to control the scroll
	 * height for the container.
	 */
	headEl = React.createRef<HTMLDivElement>()

	/**
	 * Contains the virtual rendered rows.
	 */
	bodyEl = React.createRef<HTMLDivElement>()

	/**
	 * Cache for the heights of rendered rows. This is used to provide a
	 * better estimative for render height as we render more and more rows.
	 */
	private _cachedHeight: { [key: number]: number } = {}

	constructor(props: VirtualScrollerProps) {
		super(props)
		this.handleScroll = this.handleScroll.bind(this)
		this.handleResize = this.handleResize.bind(this)
		this.state = {
			width: 0,
			height: 0,
			layout: {
				minIndex: 0,
				maxIndex: 0,
				minIndexOffset: 0,
				scroll: 0,
				totalHeight: 0,
			},
		}
	}

	shouldComponentUpdate(props: VirtualScrollerProps, state: State) {
		const currProps = this.props as any
		const nextProps = props as any
		for (const key in currProps) {
			if (currProps[key] !== nextProps[key]) {
				return true
			}
		}

		return !deepEqual(this.state, state)
	}

	componentDidMount() {
		this.startLayoutMonitor()
	}

	// This is used by the layout check to monitor changes in rendering.
	private _renderID = 0

	componentDidUpdate() {
		const rootEl = this.rootEl!
		const bodyEl = this.bodyEl.current!

		const log = DEBUG ? console.log.bind(console, '[UPDATE]') : () => {}

		// Force a layout recalculation after every render. This starts a
		// cycle of layout -> render -> layout steps until the layout reaches
		// a stable state.
		this._renderID++

		// Adjust the scrolling offset to keep the target anchor position
		const expected = this._anchor
		const actual = this.getScrollAnchor()
		log(`Expected: ${anchorToString(expected)} / Actual: ${anchorToString(actual)}`)

		// Clear any transformation from the layout before computing new
		// adjustment
		bodyEl.style.transform = ''

		if (expected && !this.adjustScroll()) {
			this.queueAdjustScroll()
			const offset = this.getScrollAdjustmentOffset(expected)
			if (Math.abs(offset) > 0.5) {
				bodyEl.style.transform = `translateY(${offset}px)`
			}
		}

		log(`After: ${anchorToString(this.getScrollAnchor())}`)

		// The scroll position can change after render (e.g. if `scrollHeight`
		// changes). We save the position so `handleScroll` will ignore the
		// scroll event.
		this._lastScrollPos = rootEl.scrollTop
	}

	componentWillUnmount() {
		clearTimeout(this._adjustScrollTimeout)
		this.stopLayoutMonitor()
	}

	render() {
		const props = this.props
		const state = this.state
		const layout = state.layout
		const classes = props.classes

		const log = DEBUG ? console.log.bind(console, '[RENDER]') : () => {}

		log(`${layoutToString(layout)}\t\t(${state.width}x${state.height})`)

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

		const items: any[] = []
		for (let i = layout.minIndex; i < layout.maxIndex; i++) {
			items.push(
				<div {...{ [DATA_INDEX_KEY]: i }} key={i} className={classes.row}>
					{props.render(i, props.data)}
				</div>
			)
		}

		return (
			<Measure client innerRef={el => (this.rootEl = el)} onResize={this.handleResize}>
				{({ measureRef }) => (
					<div
						ref={measureRef}
						className={classes.root}
						style={style}
						onScroll={this.handleScroll}
						tabIndex={-1}
					>
						<div className={classes.head} ref={this.headEl} style={{ height: layout.totalHeight }}></div>
						<div className={classes.body} ref={this.bodyEl} style={{ top: layout.minIndexOffset }}>
							{items}
						</div>
					</div>
				)}
			</Measure>
		)
	}

	/**
	 * Return the currently visible `ScrollAnchor` based entirely on the
	 * rendered state.
	 *
	 * This will return `undefined` if there is no rendered element that can
	 * be used as anchor.
	 */
	getScrollAnchor(): ScrollAnchor | undefined {
		const root = this.rootEl
		const body = this.bodyEl.current
		if (root && body) {
			// Avoid computing anything if we are at the start of the list. This
			// also handles gracefully the case we have no elements.
			if (root.scrollTop === 0) {
				return { index: 0, offset: 0, end: false }
			}

			// As a special case, we anchor to the end of the last element if
			// the scroll position is at max.
			const scrollMax = root.scrollHeight - root.clientHeight
			if (root.scrollTop === scrollMax) {
				return { index: this.props.itemCount - 1, offset: 0, end: true }
			}

			// Otherwise try to find the rendered element intersecting the
			// viewport.
			const rootRect = root.getBoundingClientRect()
			for (const el of this.getRows()) {
				const rect = el.getBoundingClientRect()
				if (rect.top <= rootRect.top && rect.bottom > rootRect.top) {
					const index = VirtualScroller.rowIndex(el)
					const offset = rect.top - rootRect.top
					return { index, offset, end: false }
				}
			}
		}
	}

	/** Return the index for a rendered row element. */
	private static rowIndex(el: Element) {
		return parseInt(el.getAttribute(DATA_INDEX_KEY)!, 10)
	}

	/** List of all currently rendered row elements. */
	private getRows(): Element[] {
		const body = this.bodyEl.current
		return body ? Array.from(body.querySelectorAll(`[${DATA_INDEX_KEY}]`)) : []
	}

	/**
	 * Target anchor for the component scroll position.
	 *
	 * This provides a stable target for the viewport that is independent of
	 * the render height, resulting in a smoother scroll behavior.
	 *
	 * This value only changes as result of user interaction (e.g. user
	 * initiated scrolling).
	 *
	 * After render the body/scroll position is adjusted to keep this anchor.
	 *
	 * Note that we don't keep this in the component state, but include it in
	 * the `LayoutInfo` to force a layout recalculation in case it changes.
	 */
	private _anchor?: ScrollAnchor = { index: 0, offset: 0, end: false }

	/**
	 * We use this to store the last scroll position and detect user initiated
	 * scrolling.
	 *
	 * Setting this after changing the root element `scrollTop` will cause
	 * the scroll event to be ignored.
	 *
	 * Note that even if the scroll event is ignored, the rendering will still
	 * be kept updated as the scroll value is monitored independently.
	 */
	private _lastScrollPos = 0

	/** Timestamp of the last user scrolling event. */
	private _lastScrollEvent?: number

	private handleScroll(ev: React.UIEvent<HTMLDivElement>) {
		const log = DEBUG ? console.log.bind(console, '[SCROLL]') : () => {}
		const target = ev.currentTarget
		const scrollPos = target.scrollTop
		if (scrollPos === this._lastScrollPos) {
			// Ignore non-user scrolling (e.g. due to changes in the rendered
			// content height).
			return
		}

		this._anchor = this.getScrollAnchor()
		this._lastScrollEvent = Date.now()

		const scrollMax = target.scrollHeight - target.clientHeight
		log(`@@@ ${scrollPos} / ${scrollMax} (${target.scrollHeight}) -- ${anchorToString(this._anchor)}`)
	}

	private _adjustScrollTimeout?: number

	private queueAdjustScroll() {
		const time = Date.now()
		const last = this._lastScrollEvent || 0
		const delta = USER_SCROLL_CORRECTION_COOLDOWN_MS - (time - last)
		const timeout = Math.min(Math.max(delta, 0), USER_SCROLL_CORRECTION_COOLDOWN_MS)
		clearTimeout(this._adjustScrollTimeout)
		this._adjustScrollTimeout = setTimeout(() => {
			this._adjustScrollTimeout = undefined
			if (!this.adjustScroll()) {
				this.queueAdjustScroll()
			}
		}, timeout)
	}

	private adjustScroll() {
		const last = this._lastScrollEvent
		const time = Date.now()
		if (last != null && time - last < USER_SCROLL_CORRECTION_COOLDOWN_MS) {
			// avoid messing up with scrolling while the user is actively
			// scrolling
			return false
		}

		const rootEl = this.rootEl
		const layout = this.state && this.state.layout
		if (!rootEl || !layout) {
			return false
		}

		// Clear any transform applied to the main body
		const bodyEl = this.bodyEl.current!
		bodyEl.style.transform = ''

		// Try to honor the scroll position given by the layout calculation.
		if (layout.scroll !== rootEl.scrollTop) {
			rootEl.scrollTop = layout.scroll
		}

		const target = this._anchor!
		let needAdjust = false
		if (target) {
			// Adjust the scrolling to keep the
			const current = this.getScrollAnchor()
			needAdjust =
				!current ||
				current.index !== target.index ||
				Math.abs(current.offset - target.offset) > ALLOWED_SCROLL_ERROR_PX
		}

		if (needAdjust) {
			// Note that scroll direction is the inverse of offset (to
			// decrease the offset we need to add to scrollTop).
			const correction = this.getScrollAdjustmentOffset(target)
			if (correction !== 0) {
				rootEl.scrollTop -= correction
			}
		}

		// Don't interpret this scroll as a user event.
		this._lastScrollPos = rootEl.scrollTop

		return true
	}

	private getScrollAdjustmentOffset(target: ScrollAnchor) {
		// Find the scroll target element and the correction offset:
		const rootEl = this.rootEl!
		const bodyEl = this.bodyEl.current!
		const itemEl = bodyEl.querySelector(`[${DATA_INDEX_KEY}="${target.index}"]`)
		if (itemEl) {
			const rootRect = rootEl.getBoundingClientRect()
			const itemRect = itemEl.getBoundingClientRect()
			const itemOffset = target.end ? itemRect.bottom - rootRect.bottom : itemRect.top - rootRect.top
			return target.offset - itemOffset
		}
		return 0
	}

	private handleResize(rect: ContentRect) {
		const size = rect.client!
		this.setState(() => ({ width: size.width, height: size.height }))
	}

	// Animation frame for the layout monitoring loop
	private _layoutMonitor?: number

	/** Start the layout monitoring loop. */
	private startLayoutMonitor() {
		this.stopLayoutMonitor()
		const doLayoutCheck = () => {
			try {
				this.checkLayout()
			} finally {
				this._layoutMonitor = window.requestAnimationFrame(doLayoutCheck)
			}
		}
		this._layoutMonitor = window.requestAnimationFrame(doLayoutCheck)
	}

	/** Stop the layout monitoring loop. */
	private stopLayoutMonitor() {
		// Clear any pending layout update
		clearTimeout(this._layoutUpdateCall)
		this._layoutUpdateCall = undefined
		// Cancel pending frame
		if (this._layoutUpdateFrame != null) {
			window.cancelAnimationFrame(this._layoutUpdateFrame)
		}
		// Stop the layout monitor callbacks
		if (this._layoutMonitor) {
			window.cancelAnimationFrame(this._layoutMonitor)
			this._layoutMonitor = undefined
		}
	}

	/** Return the current layout information used by `getLayout` */
	private getLayoutInfo(): LayoutInfo {
		return {
			itemCount: this.props.itemCount,
			width: this.state.width,
			height: this.state.height,
			scroll: this.rootEl ? this.rootEl.scrollTop : 0,
			anchor: this._anchor,
			renderID: this._renderID,
		}
	}

	// Layout update information for `checkLayout`
	private _layoutUpdateInfo?: LayoutInfo
	private _layoutUpdateLast?: number
	private _layoutUpdateCall?: number
	private _layoutUpdateFrame?: number

	/** This is called in an animation frame to check for layout changes. */
	private checkLayout() {
		const LAYOUT_DEBOUNCE_MS = 200
		const LAYOUT_THROTTLE_MS = 200

		// Check if the layout information has changed.
		const info = this.getLayoutInfo()
		if (!deepEqual(info, this._layoutUpdateInfo)) {
			// This is called with a timeout for debouncing and throttling.
			const updateLayout = () => {
				this._layoutUpdateLast = Date.now()
				this._layoutUpdateCall = undefined
				if (this._layoutUpdateFrame == null) {
					// We want to do all layout reading in an animation frame.
					this._layoutUpdateFrame = window.requestAnimationFrame(() => {
						this._layoutUpdateFrame = undefined
						const layout = this.computeLayout(this._layoutUpdateInfo!)
						this.setState(() => ({ layout }))
					})
				}
			}

			// Always use the latest layout info when updating.
			this._layoutUpdateInfo = info

			// Calls `updateLayout` debouncing and throttling:
			const time = Date.now()
			const last = this._layoutUpdateLast
			if (last == null || time - last > LAYOUT_THROTTLE_MS) {
				// consider this the first call in the batch and debounce,
				// unless it is the first.
				if (this._layoutUpdateCall == null) {
					this._layoutUpdateCall = setTimeout(updateLayout, last == null ? 0 : LAYOUT_DEBOUNCE_MS)
				}
			} else if (this._layoutUpdateCall == null) {
				// this is another call in the batch, so we apply throttling.
				const interval = LAYOUT_THROTTLE_MS - (time - last)
				this._layoutUpdateCall = setTimeout(updateLayout, interval) as number
			}
		}
	}

	/**
	 * Computes the layout for the current state of the component.
	 *
	 * Given the current scrolling position, item count and client size, this
	 * function computes:
	 *
	 * - Range of indexes (rows) to render. This includes the rows in the
	 *   visible client area and an overscan area;
	 * - Offset of the first rendered index to the list start;
	 * - Total list height;
	 * - Layout anchor.
	 *
	 * The computed layout is based on an estimation of the row heights that
	 * is refined by actually measuring rendered rows.
	 *
	 * Since heights are estimated, there will be a difference between the
	 * computed layout and the rendered layout. To account for this we:
	 *
	 * - Refine the layout calculation by measuring and caching the height of
	 *   each row as they are rendered. The actual heights are also averaged
	 *   and used as the estimate for unknown rows.
	 *
	 * - After each render the layout is recalculated to take into account the
	 *   actual measures of rendered rows. An updated layout will cause the
	 *   component to be re-rendered and the process repeats eventually reaching
	 *   a stable state.
	 *
	 *   This is taken care by including an incremental render ID in `LayoutInfo`,
	 *   which causes a layout recalculation each time it changes.
	 *
	 * - The layout anchor is used to provide a stable viewport even with the
	 *   differences between the estimated row heights and the actual rendered
	 *   row heights. The viewport adjustment happens after the component
	 *   is rendered. This avoid artifacts (random jumps and jerkyness) while
	 *   scrolling.
	 */
	private computeLayout(info: LayoutInfo): Layout {
		const log = DEBUG ? console.log.bind(console, '[LAYOUT]') : () => {}
		DEBUG_TIME && console.time('layout - new')

		const itemCount = info.itemCount

		// We try to anchor our scroll position to a visible element on the
		// screen. As we recalculate the layout, we keep this anchor so as to
		// minimize random "jumps" and jerkyness while scrolling.
		const anchor = info.anchor
		log(
			`${info.width}x${info.height} (#${itemCount} / @${info.scroll} / ${anchorToString(anchor)}) -- $${
				info.renderID
			}`
		)

		// We still use the scroll position in case we don't have a visible
		// anchor element (e.g. when first laying out or after sudden jumps
		// in the scrolling).
		//
		// Note that this is not a reliable anchor, since we estimate the row
		// heights and those change as we update the render, but it does
		// provide a good first target.
		const currentScroll = info.scroll

		// Client height available for items. We use this to determine the
		// visible limits and the overscan range.
		const clientHeight = info.height

		// Before calculating the layout, update the height of any rendered
		// rows
		for (const el of this.getRows()) {
			const index = VirtualScroller.rowIndex(el)
			const height = el.getBoundingClientRect().height
			this._cachedHeight[index] = height
			log(`item #${index + 1} has height ${height}`)
		}

		// We use the item average height to estimate both the list total
		// height and the rendered ranges. Then we repeat layout runs after
		// measuring the rendered items until we reach an equilibrium state.
		//
		// As we actually measure rendered items, this will become closer to
		// the truth and layout runs will be smoother.
		const heights = Object.values(this._cachedHeight)
		const heightEstimate = this.props.estimatedItemHeight || DEFAULT_ESTIMATED_ITEM_HEIGHT
		const avgItemHeight = heights.reduce((acc, it) => acc + it, 0) / heights.length || heightEstimate
		const totalHeight = avgItemHeight * itemCount

		const getItemHeight = (i: number) => this._cachedHeight[i] || avgItemHeight

		let scroll = currentScroll
		if (anchor) {
			// Compute the scroll position from the anchor
			scroll = 0
			for (let i = 0; i < anchor.index; i++) {
				scroll += getItemHeight(i)
			}
			if (anchor.end) {
				scroll += getItemHeight(anchor.index) - clientHeight
			}
			scroll -= anchor.offset
			scroll = Math.round(scroll)
		}

		// Overscan defines the amount of items outside the visible area that we
		// render:
		const overscanMin = this.props.overscanMinimun || DEFAULT_OVERSCAN_MINIMUM
		const overscanPages = this.props.overscanPages || DEFAULT_OVERSCAN_PAGES
		const overscan = Math.max(overscanMin, Math.round(overscanPages * clientHeight))

		// Compute the `min` and `max` pixel offsets for items to be rendered,
		// considereing the computed overscan.
		//
		// Note that is totally possible for the scroll offset to be past the
		// total height of the list. This can easily happen if we overestimate
		// the average item height and scroll to the end: as our estimative
		// is refined, the actual scroll position will be greater than our newly
		// estimated list height.
		//
		// It seems that is also possible to scroll to be negative due to
		// bouncing animations in mobile.
		//
		// In any case, clamping `min` and `max` to the actual list range avoids
		// any problems.
		const max = Math.min(totalHeight, scroll + clientHeight + overscan)
		const min = Math.max(0, Math.min(max, scroll) - overscan)

		// Find the first and last item indexes to render, based on the computed
		// `min` and `max` offsets:

		// This is the relative pixel offset of `minIndex` (the first rendered
		// item) from the beginning of the list
		let minIndexOffset = 0
		// Offset of the current item while iterating below
		let offset = 0

		// Find the first rendered index
		let minIndex = itemCount
		for (let i = 0; i < itemCount; i++) {
			const itemHeight = getItemHeight(i)
			const nextOffset = offset + itemHeight
			if (min >= offset && min < nextOffset) {
				minIndex = i
				break
			}
			minIndexOffset += itemHeight
			offset = nextOffset
		}

		// Find the next index after the last rendered item
		let maxIndex = itemCount
		for (let i = minIndex; i < itemCount; i++) {
			const itemHeight = getItemHeight(i)
			const nextOffset = offset + itemHeight
			if (max >= offset && max < nextOffset) {
				maxIndex = i + 1
				break
			}
			offset = nextOffset
		}

		DEBUG_TIME && console.timeEnd('layout - new')

		const layout = { minIndex, maxIndex, minIndexOffset, totalHeight, scroll }
		log(`RESULT: ${layoutToString(layout)}`)
		return layout
	}
}

function layoutToString(layout: Layout) {
	return (
		`🌍 #${layout.minIndex + 1}~${layout.maxIndex} ⭳${layout.minIndexOffset} ` +
		`🡙${layout.scroll}/${layout.totalHeight} `
	)
}

function anchorToString(anchor?: ScrollAnchor) {
	if (anchor) {
		const m = anchor.end ? '*' : ''
		return `⚓${anchor.index + 1}` + (anchor.offset >= 0 ? `+${anchor.offset}` : `${anchor.offset}`) + m
	} else {
		return '⚓none'
	}
}

export default withStyles(styles)(VirtualScroller)
