import {Service} from '@angular/core';
import {invoke} from "@tauri-apps/api/core";
import {ScanResultSummary} from "../models/scanner.model";

@Service()
export class ScannerService {
    // Вызов бэкенд-команды start_scan через Tauri API
    async scanDirectory(path: string): Promise<ScanResultSummary> {
        try {
            return await invoke<ScanResultSummary>('start_scan', {path});
        } catch (error) {
            console.error('Ошибка при сканировании папки в Rust:', error);
            throw error;
        }
    }
}
