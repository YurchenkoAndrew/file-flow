import {Injectable} from '@angular/core';
import {invoke} from '@tauri-apps/api/core';
import {save} from '@tauri-apps/plugin-dialog';

// T = Record<string, any> означает, что data может быть абсолютно любым объектом
export interface GenerateDocumentPayload<T = Record<string, any>> {
    templateId: string;       // Идентификатор шаблона ('application', 'memo' и т.д.)
    extension: string;        // 'pdf', 'doc' или 'docx'
    filterName: string;       // Название для диалога ОС ('PDF Документ')
    defaultFileName: string;  // Базовое имя файла, которое пользователь может поменять
    data: T;                  // Данные конкретной формы
}

@Injectable({
    providedIn: 'root'
})
export class DocumentGeneratorService {

    // Метод принимает абстрактный payload любого типа <T>
    async generateDocument<T>(payload: GenerateDocumentPayload<T>): Promise<string | null> {

        // 1. Вызываем системное окно сохранения.
        // Tauri отдаст управление ОС (Windows/macOS), где пользователь выберет путь и сможет переименовать файл.
        const savePath = await save({
            filters: [{name: payload.filterName, extensions: [payload.extension]}],
            defaultPath: payload.defaultFileName
        });

        if (!savePath) {
            return null; // Пользователь нажал "Отмена"
        }

        // 2. Отправляем единый запрос на бэкенд.
        // Rust получит data как JSON-объект и сам решит, как его обрабатывать, опираясь на template_id.
        return await invoke<string>('generate_document', {
            request: {
                template_id: payload.templateId,
                extension: payload.extension,
                save_path: savePath,
                data: payload.data
            }
        });
    }
}