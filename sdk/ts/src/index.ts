import { AlpineClient } from "@alpine-core/protocol";

export class SdkClient {
  constructor(private client: AlpineClient) {}

  static async connect(
    localAddr: string,
    remoteAddr: string,
    identity: string,
  ): Promise<SdkClient> {
    const inner = await AlpineClient.connect(localAddr, remoteAddr, identity);
    return new SdkClient(inner);
  }

  sendFrame(frame: Uint8Array) {
    return this.client.sendFrame(frame);
  }
}
