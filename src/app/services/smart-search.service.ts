import {Service} from '@angular/core';
import {invoke} from "@tauri-apps/api/core";

export interface SearchResultDto {
    id: number;
    file_path: string;
    snippet: string;
    score: number;
}

@Service()
export class SmartSearchService {
    // Вызов нашей Rust Tauri-команды умного поиска
    async search(query: string): Promise<SearchResultDto[]> {
        try {
            return await invoke<SearchResultDto[]>('smart_search_command', {
                query: query
            });
        } catch (error) {
            console.error('Ошибка при выполнении семантического поиска в Rust:', error);
            throw error;
        }
    }

    // Запуск нейросканирования папки (используем уже существующую бэкенд-команду)
    async startNeuralScan(targetPath: string): Promise<number> {
        try {
            return await invoke<number>('start_neural_scan', {
                targetPath: targetPath
            });
        } catch (error) {
            console.error('Ошибка при запуске нейросканирования:', error);
            throw error;
        }
    }

    // Открытие файла / папки через shared-сервис (reveal_file_in_folder)
    async revealInFolder(filePath: string): Promise<void> {
        try {
            await invoke('reveal_file_in_folder', {path: filePath});
        } catch (error) {
            console.error('Не удалось открыть файл в проводнике:', error);
            throw error;
        }
    }

    async getWatchedFolders(): Promise<string[]> {
        return await invoke('get_watched_folders_command');
    }

    async removeWatchedFolder(path: string): Promise<void> {
        await invoke('remove_watched_folder_command', {path});
    }
}
