export const cloudPortOffset: number;
export const cloudTestPorts: Readonly<{
  preview: number; auth: number; firestore: number;
  websocket: number; hub: number; logging: number;
}>;
export const cloudTestOrigins: Readonly<{ preview: string; auth: string; firestore: string }>;
