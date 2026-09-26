import type { ReactNode, CSSProperties } from 'react';
/** Appears when settings are dirty. */
export interface SaveBarProps { message: ReactNode; onRevert?: () => void; onSave?: () => void; saveDisabled?: boolean; floating?: boolean; style?: CSSProperties; }
export declare function SaveBar(props: SaveBarProps): JSX.Element;
