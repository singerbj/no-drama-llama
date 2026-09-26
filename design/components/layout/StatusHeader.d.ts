import type { ReactNode, CSSProperties } from 'react';
import type { Tone } from '../core/StatusDot';
/** Window header showing the tray state. */
export interface StatusHeaderProps { tone?: Tone; title: ReactNode; subtitle?: ReactNode; actions?: ReactNode; style?: CSSProperties; }
export declare function StatusHeader(props: StatusHeaderProps): JSX.Element;
