import type { ReactNode, CSSProperties } from 'react';
import type { Tone } from '../core/StatusDot';
/** Inline notice banner tinted by a status tone. */
export interface BannerProps { tone?: Tone; action?: ReactNode; style?: CSSProperties; children?: ReactNode; }
export declare function Banner(props: BannerProps): JSX.Element;
