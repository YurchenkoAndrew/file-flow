import {Component, signal} from '@angular/core';
import {MatButtonModule} from "@angular/material/button";
import {MatCardModule} from "@angular/material/card";
import {MatInputModule} from "@angular/material/input";
import {MatIconModule} from "@angular/material/icon";
import {MatProgressBarModule} from "@angular/material/progress-bar";
import {MatRadioModule} from "@angular/material/radio";
import {FormsModule} from "@angular/forms";
import {CommonModule} from "@angular/common";
import {MatFormFieldModule} from "@angular/material/form-field";
import {open} from '@tauri-apps/plugin-dialog';

@Component({
    selector: 'app-sorter',
    imports: [
        CommonModule,
        FormsModule,
        MatCardModule,
        MatFormFieldModule,
        MatInputModule,
        MatButtonModule,
        MatRadioModule,
        MatProgressBarModule,
        MatIconModule
    ],
    templateUrl: './sorter.html',
    styleUrl: './sorter.css',
})
export class Sorter {

    title = 'File Flow';

    // Пути и настройки (объявлены как сигналы для мгновенного обновления интерфейса)
    sourcePath = signal<string>('');
    destinationPath = signal<string>('');
    operationMode: string = 'copy'; // 'copy' или 'move'

    // Состояние процесса
    isProcessing = signal<boolean>(false);
    progressValue = signal<number>(0);
    statusMessage = signal<string>('Готов к работе');

    // Выбор папки-источника через нативный диалог Tauri
    async selectSource() {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: 'Выберите папку-источник'
            });
            if (selected && typeof selected === 'string') {
                this.sourcePath.set(selected);
            }
        } catch (error) {
            console.error('Ошибка выбора папки-источника:', error);
        }
    }

    // Выбор папки-назначения через нативный диалог Tauri
    async selectDestination() {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: 'Выберите папку назначения'
            });
            if (selected && typeof selected === 'string') {
                this.destinationPath.set(selected);
            }
        } catch (error) {
            console.error('Ошибка выбора папки назначения:', error);
        }
    }

    // Запуск сортировки
    startSorting() {
        if (!this.sourcePath() || !this.destinationPath()) {
            alert('Пожалуйста, выберите папки источника и назначения!');
            return;
        }

        this.isProcessing.set(true);
        this.statusMessage.set('Сканирование и сортировка файлов...');
        this.progressValue.set(30); // Демо-прогресс
    }
}