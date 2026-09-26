import type { CSSProperties } from 'react';
/** Settings-window tab list. */
export interface SideNavProps { items: { id: string; title: string }[]; active?: string; onSelect?: (id: string) => void; style?: CSSProperties; }
export declare function SideNav(props: SideNavProps): JSX.Element;
