/** One downloadable model in "Download a model". */
export interface CatalogRowProps {
  label: string;
  /** "fits your GPU", "partly in RAM - slow", "too big for this PC" */
  note: string;
  fit?: 'gpu' | 'gpuAndRam' | 'tooBig';
  recommended?: boolean;
  state?: 'download' | 'installed' | 'downloading' | 'unavailable';
  onAction?: () => void;
  last?: boolean;
}
export declare function CatalogRow(props: CatalogRowProps): JSX.Element;
