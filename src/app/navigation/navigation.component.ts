import {Component, inject} from '@angular/core';
import {toSignal} from '@angular/core/rxjs-interop';
import {BreakpointObserver, Breakpoints} from '@angular/cdk/layout';
import {MatToolbarModule} from '@angular/material/toolbar';
import {MatButtonModule} from '@angular/material/button';
import {MatSidenavModule} from '@angular/material/sidenav';
import {MatListModule} from '@angular/material/list';
import {MatIconModule} from '@angular/material/icon';
import {map} from 'rxjs/operators';
import {RouterLink, RouterLinkActive, RouterOutlet} from "@angular/router";
import {Theme} from "../services/theme";
import {MatTooltip} from "@angular/material/tooltip";
import {MatSlideToggle} from "@angular/material/slide-toggle";

@Component({
    selector: 'app-navigation',
    templateUrl: './navigation.component.html',
    styleUrl: './navigation.component.css',
    imports: [MatToolbarModule, MatButtonModule, MatSidenavModule, MatListModule, MatIconModule, RouterOutlet, RouterLink, RouterLinkActive, MatTooltip, MatSlideToggle],
})
export class NavigationComponent {
    private readonly breakpointObserver = inject(BreakpointObserver);
    // Внедряем сервис тем
    public readonly themeService = inject(Theme);

    readonly isHandset = toSignal(
        this.breakpointObserver.observe(Breakpoints.Handset).pipe(map((result) => result.matches)),
        {initialValue: false},
    );
}
