import type { CSSProperties, ReactNode } from 'react';
import type { Tone } from '../core/StatusDot';
/**
 * Click-through on-screen popup shown when the model pauses or resumes.
 * @startingPoint section="Feedback" subtitle="On-screen pause/resume popup" viewport="700x300"
 */
export interface StatusPopupProps { tone?: Tone; title: ReactNode; subtitle?: ReactNode; /** loop the site's fade-in/out demo */ animated?: boolean; style?: CSSProperties; }
export declare function StatusPopup(props: StatusPopupProps): JSX.Element;
