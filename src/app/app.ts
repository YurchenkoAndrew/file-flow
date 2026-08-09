import {Component, signal} from '@angular/core';
import {NavigationComponent} from "./navigation/navigation.component";

@Component({
    selector: 'app-root',
  imports: [NavigationComponent],
    templateUrl: './app.html',
    styleUrl: './app.css'
})
export class App {
    protected readonly title = signal('File flow');
    constructor() {
        // Отключаем стандартное контекстное меню во всем приложении
        document.addEventListener('contextmenu', (event) => {
            event.preventDefault();
        });
    }
}
