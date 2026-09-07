import {Component, inject} from '@angular/core';
import {CommonModule} from "@angular/common";
import {MatCardModule} from "@angular/material/card";
import {MatButtonModule} from "@angular/material/button";
import {MatIconModule} from "@angular/material/icon";
import {MatList, MatListItem} from "@angular/material/list";
import {UpdaterService} from "../../services/updater.service";
import {MatDialog} from "@angular/material/dialog";
import {UpdaterDialog} from "../updater-dialog/updater-dialog";

@Component({
    selector: 'app-about',
    imports: [
        CommonModule,
        MatCardModule,
        MatButtonModule,
        MatIconModule,
        MatList,
        MatListItem
    ],
    templateUrl: './about.html',
    styleUrl: './about.css',
})
export class About {
    updater = inject(UpdaterService);
    private dialog = inject(MatDialog);

    // Открываем диалог скачивания, когда пользователь сам нажал кнопку на странице
    openUpdateDialog() {
        this.dialog.open(UpdaterDialog, {
            width: '400px',
            disableClose: true
        });
    }
}
