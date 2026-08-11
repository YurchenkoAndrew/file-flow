import { Service } from '@angular/core';
import {invoke} from "@tauri-apps/api/core";

@Service()
export class SharedService {
    async showFileByPath(path: string): Promise<void> {
        // Выводим для проверки
        console.log('Отправляем путь в Rust:', path);

        try {
            // Имя метода в invoke должно точно совпадать с именем функции в Rust
            // (или в camelCase/snake_case, Tauri старается их сопоставлять, но лучше писать как в Rust или в camelCase)
            await invoke('reveal_file_in_folder', { path: path });
        } catch (error) {
            console.error('Ошибка при вызове Rust команды:', error);
        }
    }
}
