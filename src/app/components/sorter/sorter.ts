import {Component, inject, OnInit, signal} from '@angular/core';
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
import {StateService} from "../../services/state.service";
import {SorterService} from "../../services/sorter";
import {SorterOptions} from "../../models/sorter.model";

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
export class Sorter implements OnInit {

    title = 'Сортировщик';
    private readonly sharedState = inject(StateService);
    private sorterService = inject(SorterService);

    // Пути и настройки (объявлены как сигналы для мгновенного обновления интерфейса)
    sourcePath = signal<string>('');
    destinationPath = signal<string>('');
    operationMode: string = 'copy'; // 'copy' или 'move'

    // Состояние процесса
    isProcessing = signal<boolean>(false);
    progressValue = signal<number>(0);
    statusMessage = signal<string>('Готов к работе');

    ngOnInit() {
        // Если поле источника пустое, но в сервисе есть сохраненный путь от сканера — подставляем его
        if (!this.sourcePath() && this.sharedState.lastScannedPath()) {
            this.sourcePath.set(this.sharedState.lastScannedPath());
            this.statusMessage.set('Источник автоматически подставлен из сканера');
        }
    }

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
    async startSorting() {
        if (!this.sourcePath() || !this.destinationPath()) {
            alert('Пожалуйста, выберите папки источника и назначения!');
            return;
        }

        this.isProcessing.set(true);
        this.statusMessage.set('Выполняется сортировка файлов...');
        this.progressValue.set(30);

        try {
            const options: SorterOptions = {
                source_path: this.sourcePath(),
                target_directory: this.destinationPath(),
                copy_files: this.operationMode === 'copy',
                group_by_year: true
            };

            const result = await this.sorterService.startSorting(options);

            this.progressValue.set(100);
            this.statusMessage.set(`Готово! Успешно: ${result.success_count}, ошибок: ${result.error_count}`);

            if (result.error_count > 0) {
                console.warn('Ошибки при сортировке:', result.errors);
            }
        } catch (error) {
            this.statusMessage.set(`Ошибка при сортировке: ${error}`);
        } finally {
            this.isProcessing.set(false);
        }
    }
}