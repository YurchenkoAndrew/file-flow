import { ComponentFixture, TestBed } from '@angular/core/testing';

import { UpdaterDialog } from './updater-dialog';

describe('UpdaterDialog', () => {
  let component: UpdaterDialog;
  let fixture: ComponentFixture<UpdaterDialog>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [UpdaterDialog],
    }).compileComponents();

    fixture = TestBed.createComponent(UpdaterDialog);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
