export interface NeuroScanStatus {
    is_running: boolean;
    processed: number;
    total: number;
    current_folder?: string | null;
}