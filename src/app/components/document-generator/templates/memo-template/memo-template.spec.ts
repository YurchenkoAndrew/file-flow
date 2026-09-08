import { ComponentFixture, TestBed } from '@angular/core/testing';

import { MemoTemplate } from './memo-template';

describe('MemoTemplate', () => {
  let component: MemoTemplate;
  let fixture: ComponentFixture<MemoTemplate>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [MemoTemplate],
    }).compileComponents();

    fixture = TestBed.createComponent(MemoTemplate);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
