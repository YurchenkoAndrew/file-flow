import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ApplicationTemplate } from './application-template';

describe('ApplicationTemplate', () => {
  let component: ApplicationTemplate;
  let fixture: ComponentFixture<ApplicationTemplate>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ApplicationTemplate],
    }).compileComponents();

    fixture = TestBed.createComponent(ApplicationTemplate);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
