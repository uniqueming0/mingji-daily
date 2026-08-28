// mammoth（docx 文本提取）的类型声明
// 说明：npm 上无 @types/mammoth 包，mammoth 本身不含完整类型，
// 这里按本项目实际用到的 API 做最小声明。
declare module "mammoth" {
  interface MammothOptions {
    arrayBuffer: ArrayBuffer;
  }

  interface MammothResult {
    value: string;
    messages: unknown[];
  }

  interface Mammoth {
    extractRawText(options: MammothOptions): Promise<MammothResult>;
  }

  const mammoth: Mammoth;
  export default mammoth;
}
