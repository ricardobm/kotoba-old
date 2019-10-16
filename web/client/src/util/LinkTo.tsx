import React from 'react'
import { Link as RouterLink, LinkProps as RouterLinkProps } from 'react-router-dom'
import Link, { LinkProps } from '@material-ui/core/Link'
import { LocationDescriptor } from 'history'

const InnerLink = React.forwardRef<HTMLAnchorElement, RouterLinkProps>((props, ref) => (
	<RouterLink innerRef={ref} {...props} />
))

interface InnerLinkProps extends LinkProps {
	to: LocationDescriptor
}

export default function LinkTo(props: InnerLinkProps) {
	return <Link component={InnerLink} {...props} />
}
