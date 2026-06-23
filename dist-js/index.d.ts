export interface Assets {
    largeImage?: string;
    largeText?: string;
    /** Makes the large image a clickable link. */
    largeUrl?: string;
    smallImage?: string;
    smallText?: string;
    /** Makes the small image a clickable link. */
    smallUrl?: string;
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
    /** Activity name. Discord may substitute the registered app name for an app RPC. */
    name?: string;
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
/** Send a raw activity payload straight to Discord (bypasses the typed API). Advanced/experimental. */
export declare function setActivityRaw(payload: unknown): Promise<void>;
export declare function clearActivity(): Promise<void>;
export declare function isConnected(): Promise<boolean>;
export declare function getCurrentUser(): Promise<User | null>;
