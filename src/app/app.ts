import {Component, inject, OnInit, signal} from '@angular/core';
import {NavigationComponent} from "./navigation/navigation.component";
import {UpdaterService} from "./services/updater.service";

@Component({
    selector: 'app-root',
    imports: [NavigationComponent],
    templateUrl: './app.html',
    styleUrl: './app.css'
})
export class App implements OnInit {
    protected readonly title = signal('File flow');
    // Внедряем сервис обновлений через Angular inject
    updater = inject(UpdaterService);

    constructor() {
        // Отключаем стандартное контекстное меню во всем приложении
        document.addEventListener('contextmenu', (event) => {
            event.preventDefault();
        });
    }

    ngOnInit(): void {
        // 1. Проверяем при старте (через 3 секунды после запуска)
        setTimeout(() => {
            this.updater.checkForUpdates(true).then();
        }, 3000);

        // 2. Настраиваем периодическую проверку в фоне
        // 4 часа = 4 * 60 минут * 60 секунд * 1000 миллисекунд = 14 400 000 мс
        const CHECK_INTERVAL = 4 * 60 * 60 * 1000;

        setInterval(() => {
            // Если обновление уже найдено и висит плашка, лишний раз сеть не дергаем
            if (!this.updater.updateAvailable()) {
                this.updater.checkForUpdates(true).then();
            }
        }, CHECK_INTERVAL);
    }
}
