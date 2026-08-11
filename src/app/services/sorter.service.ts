import {Service} from '@angular/core';
import {invoke} from "@tauri-apps/api/core";
import {SorterOptions, SortResultSummary} from "../models/sorter.model";

@Service()
export class SorterService {

    // Вызов бэкенд-команды start_sorting через Tauri API
    async startSorting(options: SorterOptions): Promise<SortResultSummary> {
        try {
            return await invoke<SortResultSummary>('start_sorting', {options});
        } catch (error) {
            console.error('Ошибка при сортировке файлов в Rust:', error);
            throw error;
        }
    }
}
