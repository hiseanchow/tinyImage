export type NotifyMode = 'dialog' | 'notification' | 'silent'
export type OutputMode = 'alongside' | 'overwrite' | 'directory'
export type Theme = 'auto' | 'light' | 'dark'

export interface AppSettings {
  apiKey: string
  notifyMode: NotifyMode
  outputMode: OutputMode
  outputDirectory: string
  contextMenuEnabled: boolean
  theme: Theme
}

export type FileStatus = 'pending' | 'compressing' | 'done' | 'error'

export interface FileItem {
  id: string
  path: string
  name: string
  originalSize: number
  compressedSize: number
  status: FileStatus
  errorMessage?: string
  outputPath?: string
}

export interface CompressResult {
  input_size: number
  output_size: number
  output_path: string
}
