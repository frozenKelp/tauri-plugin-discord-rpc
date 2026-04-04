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
export interface Activity {
    state?: string;
    details?: string;
    assets?: Assets;
    buttons?: Button[];
    party?: Party;
    timestamps?: Timestamps;
}
export declare function connect(appId: string): Promise<void>;
export declare function disconnect(): Promise<void>;
export declare function setActivity(payload: Activity): Promise<void>;
export declare function clearActivity(): Promise<void>;
export declare function isRunning(): Promise<boolean>;
