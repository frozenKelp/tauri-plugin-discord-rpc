export interface Assets {
    largeImage?: string;
    largeText?: string;
    smallImage?: string;
    smallText?: string;
}
export interface Button {
    label: string;
    url: string;
}
export interface Party {
    id?: string;
    currentSize?: number;
    maxSize?: number;
}
export interface Timestamps {
    start?: number;
    end?: number;
}
export interface User {
    id: string;
    username: string;
    discriminator?: string;
    globalName?: string;
    avatar?: string;
}
export interface Activity {
    state?: string;
    details?: string;
    /** Makes the state line a clickable link. */
    stateUrl?: string;
    /** Makes the details line a clickable link. */
    detailsUrl?: string;
    assets?: Assets;
    buttons?: Button[];
    party?: Party;
    timestamps?: Timestamps;
    /** 0 Playing, 2 Listening, 3 Watching, 5 Competing. */
    activityType?: number;
    /** 0 Name, 1 State, 2 Details — compact headline. */
    statusDisplayType?: number;
}
export declare function connect(appId: string): Promise<void>;
export declare function disconnect(): Promise<void>;
export declare function setActivity(payload: Activity): Promise<void>;
export declare function clearActivity(): Promise<void>;
export declare function isConnected(): Promise<boolean>;
export declare function getCurrentUser(): Promise<User | null>;
