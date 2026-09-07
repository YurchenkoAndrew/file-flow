import {Component, inject} from '@angular/core';
import {
  MatDialogActions,
  MatDialogClose,
  MatDialogContent,
  MatDialogRef,
  MatDialogTitle
} from "@angular/material/dialog";
import {MatIcon} from "@angular/material/icon";
import {MatProgressBar} from "@angular/material/progress-bar";
import {MatButton} from "@angular/material/button";
import {UpdaterService} from "../../services/updater.service";
import {relaunch} from "@tauri-apps/plugin-process";
import {DownloadEvent} from "@tauri-apps/plugin-updater";

@Component({
    selector: 'app-updater-dialog',
  imports: [
    MatDialogTitle,
    MatIcon,
    MatDialogContent,
    MatProgressBar,
    MatDialogActions,
    MatButton,
    MatDialogClose
  ],
    templateUrl: './updater-dialog.html',
    styleUrl: './updater-dialog.css',
})
export class UpdaterDialog {
    updater = inject(UpdaterService);
    dialogRef = inject(MatDialogRef<UpdaterDialog>);

    isDownloading = false;

    async startUpdate() {
        this.isDownloading = true;

        try {
            const updateObject = this.updater.currentUpdate;

            if (!updateObject) {
                this.isDownloading = false;
                return;
            }

            this.updater.statusMessage.set('Загрузка обновления...');
            let downloaded = 0;
            let contentLength = 0;

            // Используем строгий тип DownloadEvent из плагина Tauri
            await updateObject.downloadAndInstall((event: DownloadEvent) => {
                switch (event.event) {
                    case 'Started':
                        contentLength = event.data.contentLength || 0;
                        break;
                    case 'Progress':
                        downloaded += event.data.chunkLength;
                        if (contentLength > 0) {
                            const percent = Math.round((downloaded / contentLength) * 100);
                            this.updater.downloadProgress.set(percent);
                            this.updater.statusMessage.set(`Загрузка: ${percent}%`);
                        }
                        break;
                    case 'Finished':
                        this.updater.statusMessage.set('Установка завершена. Перезапуск...');
                        break;
                }
            });

            this.dialogRef.close();
            await relaunch();

        } catch (error) {
            console.error('Ошибка при обновлении:', error);
            this.updater.statusMessage.set('Ошибка загрузки обновления');
            this.isDownloading = false;
        }
    }
}
