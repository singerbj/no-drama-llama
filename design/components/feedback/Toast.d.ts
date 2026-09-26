import type { ReactNode, CSSProperties } from 'react';
/** Short confirmation ("Saved") or error ("Not saved: ..."). */
export interface ToastProps { error?: boolean; floating?: boolean; style?: CSSProperties; children?: ReactNode; }
export declare function Toast(props: ToastProps): JSX.Element;
