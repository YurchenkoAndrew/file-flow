import {Service} from '@angular/core';
import {DuplicateGroup} from "../models/scanner.model";
import {invoke} from "@tauri-apps/api/core";

// Описываем структуру ответа, чтобы TypeScript понимал, что возвращает Rust
export interface CleanupResponse {
    count: number;
    freed_space: number;
}

@Service()
export class DuplicatesService {
    // Теперь метод принимает ID сессии и возвращает объект CleanupResponse
    async removeDuplicates(sessionId: number, groups: DuplicateGroup[]): Promise<CleanupResponse> {
        try {
            // Имена ключей передаваемого объекта конвертируются для Rust:
            // sessionId на фронте станет session_id на бэкенде.
            return await invoke<CleanupResponse>('clean_duplicates_command', {
                sessionId: sessionId,
                groups: groups
            });
        } catch (error) {
            console.error('Ошибка при удалении дубликатов:', error);
            throw error;
        }
    }
}
