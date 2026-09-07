import {Service, signal} from '@angular/core';
import {check, Update} from "@tauri-apps/plugin-updater";
import {relaunch} from "@tauri-apps/plugin-process";

@Service()
export class UpdaterService {
    // Сигналы для отслеживания состояния в UI
    isChecking = signal(false);
    updateAvailable = signal(false);
    updateVersion = signal('');
    downloadProgress = signal(0);
    statusMessage = signal('');
    currentUpdate: any = null;

    // Метод проверки обновлений
    async checkForUpdates(silent: boolean = false) {
        try {
            // check() вернет объект обновления (Update) или null
            const update = await check();

            // Просто проверяем, вернулся ли объект
            if (update) {
                this.currentUpdate = update;
                this.updateVersion.set(update.version);
                this.updateAvailable.set(true);
            } else {
                this.updateAvailable.set(false);
            }
        } catch (error) {
            console.error('Ошибка проверки обновлений:', error);
        }
    }

    // Приватный метод скачивания
    private async downloadAndInstallUpdate(update: Update) {
        try {
            this.statusMessage.set('Загрузка обновления...');
            let downloaded = 0;
            let contentLength = 0;

            await update.downloadAndInstall((event) => {
                switch (event.event) {
                    case 'Started':
                        contentLength = event.data.contentLength || 0;
                        console.log(`Начало загрузки. Размер: ${contentLength} байт`);
                        break;
                    case 'Progress':
                        downloaded += event.data.chunkLength;
                        if (contentLength > 0) {
                            const percent = Math.round((downloaded / contentLength) * 100);
                            this.downloadProgress.set(percent);
                            this.statusMessage.set(`Загрузка: ${percent}%`);
                        }
                        break;
                    case 'Finished':
                        this.statusMessage.set('Установка завершена. Перезапуск...');
                        console.log('Загрузка завершена!');
                        break;
                }
            });

            // Перезапускаем приложение для применения новой версии
            await relaunch();
        } catch (error) {
            console.error('Ошибка при скачивании обновления:', error);
            this.statusMessage.set('Ошибка загрузки обновления');
        }
    }
}
