import { Component } from '@angular/core';
import {CommonModule} from "@angular/common";
import {MatCardModule} from "@angular/material/card";
import {MatButtonModule} from "@angular/material/button";
import {MatIconModule} from "@angular/material/icon";
import {RouterLink} from "@angular/router";
import {MatList, MatListItem} from "@angular/material/list";

@Component({
  selector: 'app-about',
  imports: [
    CommonModule,
    MatCardModule,
    MatButtonModule,
    MatIconModule,
    RouterLink,
    MatList,
    MatListItem
  ],
  templateUrl: './about.html',
  styleUrl: './about.css',
})
export class About {}
