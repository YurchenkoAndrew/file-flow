import {Service, signal} from '@angular/core';
import {ScanResultSummary} from "../models/scanner.model";

@Service()
export class StateService {
    // Храним ID активной сессии и сам путь
    readonly currentSessionId = signal<number | null>(null);
    readonly activePath = signal<string>('');

    // Храним сам результат сканирования перманентно
    readonly currentScanResult = signal<ScanResultSummary | null>(null);

    // Удобный метод для установки активной сессии после сканирования
    setActiveSession(sessionId: number, path: string, result: ScanResultSummary) {
        this.currentSessionId.set(sessionId);
        this.activePath.set(path);
        this.currentScanResult.set(result);
    }

    clearSession() {
        this.currentSessionId.set(null);
        this.activePath.set('');
        this.currentScanResult.set(null);
    }
}
