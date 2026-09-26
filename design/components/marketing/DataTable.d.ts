import type { ReactNode, CSSProperties } from 'react';
/** Marketing/docs table. */
export interface DataTableProps {
  caption?: ReactNode;
  columns: ReactNode[];
  rows: ReactNode[][];
  style?: CSSProperties;
}
export declare function DataTable(props: DataTableProps): JSX.Element;
