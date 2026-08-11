import { Service } from '@angular/core';
import {DuplicateGroup} from "../models/scanner.model";
import {invoke} from "@tauri-apps/api/core";

@Service()
export class DuplicatesService {
    async removeDuplicates(groups: DuplicateGroup[]): Promise<[number, number]> {
        try {
            return await invoke<[number, number]>('remove_duplicates', { groups });
        } catch (error) {
            console.error('Ошибка при удалении дубликатов:', error);
            throw error;
        }
    }
}
