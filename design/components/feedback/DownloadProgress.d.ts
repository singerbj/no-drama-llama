import type { CSSProperties } from 'react';
/** Model / Laya download progress card. */
export interface DownloadProgressProps { label: string; done?: number; total?: number; onCancel?: () => void; note?: string; style?: CSSProperties; }
export declare function DownloadProgress(props: DownloadProgressProps): JSX.Element;
